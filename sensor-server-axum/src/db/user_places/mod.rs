use diesel::prelude::*;
use sensor_lib::api::model::api_id::ApiId;

use crate::{
    db::{DbConn, Error},
    model::{self, NewUserPlace, UserPlace},
};

pub fn insert_user_place(conn: &mut DbConn, place: NewUserPlace) -> Result<UserPlace, Error> {
    use crate::schema::user_places::dsl::user_places as user_places_table;

    let place: Vec<UserPlace> = place.insert_into(user_places_table).load(conn)?;

    place
        .first()
        .ok_or(Error::NotFound("The returned vec was empty".into()))
        .map(|e| e.clone())
}

pub enum UserPlaceIdentifier {
    UserApiId(ApiId),
    PlaceApiId(ApiId),
}

pub fn get_user_place(
    conn: &mut DbConn,
    identifier: UserPlaceIdentifier,
) -> Result<Vec<UserPlace>, Error> {
    match identifier {
        UserPlaceIdentifier::UserApiId(api_id) => {
            use crate::schema::user_places::dsl::user_places as user_places_table;
            use crate::schema::{users::dsl as user, users::dsl::users as users_table};

            let res = users_table
                .filter(user::api_id.eq(api_id.as_str()))
                .inner_join(user_places_table)
                .select(model::UserPlace::as_select())
                .load(conn)?;

            Ok(res)
        }
        UserPlaceIdentifier::PlaceApiId(api_id) => {
            use crate::schema::{
                user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
            };

            let res = user_places_table
                .filter(user_place::api_id.eq(api_id.as_str()))
                .select(model::UserPlace::as_select())
                .load(conn)?;

            Ok(res)
        }
    }
}

pub fn delete_user_place(
    conn: &mut DbConn,
    identifier: UserPlaceIdentifier,
) -> Result<Vec<UserPlace>, Error> {
    match identifier {
        UserPlaceIdentifier::UserApiId(api_id) => {
            use crate::schema::user_places::dsl::{user_id, user_places};
            use crate::schema::users::dsl::{api_id as user_api_id, id as user_pk, users};

            // find the primary key of the user from their API ID.
            let target_user_id: i32 = users
                .filter(user_api_id.eq(api_id.as_str()))
                .select(user_pk)
                .first(conn)?;

            let deleted_places =
                diesel::delete(user_places.filter(user_id.eq(target_user_id))).get_results(conn)?;

            Ok(deleted_places)
        }
        UserPlaceIdentifier::PlaceApiId(api_id) => {
            use crate::schema::user_places::dsl::{api_id as place_api_id, user_places};

            let deleted_places =
                diesel::delete(user_places.filter(place_api_id.eq(api_id.as_str())))
                    .get_results(conn)?;

            Ok(deleted_places)
        }
    }
}

#[cfg(test)]
mod tests {
    use sensor_lib::api::model::api_id::ApiId;

    use crate::{
        db::{
            tests::{create_test_user, create_test_user_place, establish_test_connection},
            user_places::{get_user_place, insert_user_place},
        },
        model::NewUserPlace,
    };

    use super::*;

    #[test]
    fn test_place_connection() {
        let _conn = establish_test_connection();
    }

    #[test]
    fn test_new_place() {
        let mut conn = establish_test_connection();
        let user = create_test_user(&mut conn);

        let new_up: NewUserPlace = NewUserPlace {
            api_id: ApiId::random().to_string(),
            user_id: user.id,
            name: "new_testuserplace".to_string(),
            description: Some("le description".to_string()),
            color_id: 1,
        };

        let i_up = insert_user_place(&mut conn, new_up.clone()).expect("No errors expected");

        assert_eq!(i_up.api_id, new_up.api_id);
        assert_eq!(i_up.user_id, new_up.user_id);
        assert_eq!(i_up.name, new_up.name);
        assert_eq!(i_up.description, new_up.description);
        assert_eq!(i_up.color_id, new_up.color_id);
    }

    #[test]
    fn test_get_place() {
        let mut conn = establish_test_connection();

        let user = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);

        let _p1 = get_user_place(
            &mut conn,
            UserPlaceIdentifier::PlaceApiId(
                ApiId::from_string(&place.api_id).expect("ApiId valid"),
            ),
        )
        .expect("No error");

        let _p2 = get_user_place(
            &mut conn,
            UserPlaceIdentifier::UserApiId(ApiId::from_string(&user.api_id).expect("ApiId valid")),
        )
        .expect("No errror");
    }

    #[test]
    fn test_delete_place() {
        let mut conn = establish_test_connection();
        let user = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);
        let place_api_id = ApiId::from_string(&place.api_id).expect("API ID should be valid");

        let deleted_places = delete_user_place(
            &mut conn,
            UserPlaceIdentifier::PlaceApiId(place_api_id.clone()),
        )
        .expect("Delete operation should not fail");

        assert_eq!(deleted_places.len(), 1);
        assert_eq!(deleted_places[0].id, place.id);
        assert_eq!(deleted_places[0].name, place.name);

        use crate::schema::user_places::dsl::{id, user_places};

        let find_result = user_places
            .filter(id.eq(place.id))
            .select(id) // We only need to know if it exists, so just select the id
            .first::<i32>(&mut conn)
            .optional(); // .optional() makes it return Ok(None) instead of Err(NotFound)

        assert_eq!(
            find_result.expect("DB query should not fail"),
            None,
            "The place with id {} should not be found after deletion.",
            place.id
        );
    }
}
