use diesel::prelude::*;

use crate::{
    db::{DbConn, Error},
    model::{self, NewUserPlace, UserPlace},
};
#[derive(Debug)]
pub enum Identifier {
    UserId(i32),
    PlaceNameAndUserId(String, i32),
}

pub fn get_user_place_id(conn: &mut DbConn, identifier: Identifier) -> Result<Vec<i32>, Error> {
    use crate::schema::{
        user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
    };

    let r = match identifier {
        Identifier::UserId(id) => user_places_table
            .filter(user_place::user_id.eq(id))
            .select(user_place::id)
            .load::<i32>(conn)?,
        Identifier::PlaceNameAndUserId(name, user_id) => user_places_table
            .filter(user_place::name.eq(name))
            .filter(user_place::user_id.eq(user_id))
            .select(user_place::id)
            .load::<i32>(conn)?,
    };

    Ok(r)
}

pub fn insert_user_place(conn: &mut DbConn, place: NewUserPlace) -> Result<UserPlace, Error> {
    use crate::schema::user_places::dsl::user_places as user_places_table;

    let place: Vec<UserPlace> = place.insert_into(user_places_table).load(conn)?;

    place
        .first()
        .ok_or(Error::NotFound("The returned vec was empty".into()))
        .map(|e| e.clone())
}

pub fn get_user_place(conn: &mut DbConn, identifier: Identifier) -> Result<Vec<UserPlace>, Error> {
    match identifier {
        Identifier::UserId(id) => {
            use crate::schema::{
                user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
            };

            let res = user_places_table
                .filter(user_place::user_id.eq(id))
                .select(model::UserPlace::as_select())
                .load(conn)?;

            Ok(res)
        }
        Identifier::PlaceNameAndUserId(name, user_id) => {
            use crate::schema::{
                user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
            };

            let res = user_places_table
                .filter(user_place::name.eq(name))
                .filter(user_place::user_id.eq(user_id))
                .select(model::UserPlace::as_select())
                .load(conn)?;

            Ok(res)
        }
    }
}

pub fn delete_user_place(
    conn: &mut DbConn,
    identifier: Identifier,
) -> Result<Vec<UserPlace>, Error> {
    use crate::schema::{
        user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
    };

    match identifier {
        Identifier::UserId(id) => {
            let deleted_places =
                diesel::delete(user_places_table.filter(user_place::user_id.eq(id)))
                    .get_results(conn)?;

            Ok(deleted_places)
        }
        Identifier::PlaceNameAndUserId(name, user_id) => {
            let deleted_places = diesel::delete(
                user_places_table
                    .filter(user_place::name.eq(name))
                    .filter(user_place::user_id.eq(user_id)),
            )
            .get_results(conn)?;

            if deleted_places.len() < 1 {
                Err(Error::NotFound("No places deleted by place id".into()))
            } else {
                Ok(deleted_places)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        db::{
            establish_connection,
            tests::{create_test_user, create_test_user_place},
            user_places::{get_user_place, insert_user_place},
        },
        model::NewUserPlace,
    };

    use super::*;

    #[test]
    fn test_new_place() {
        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);

        let new_up: NewUserPlace = NewUserPlace {
            user_id: user.id,
            name: "new_testuserplace".to_string(),
            description: Some("le description".to_string()),
            color_id: 1,
        };

        let i_up = insert_user_place(&mut conn, new_up.clone()).expect("No errors expected");

        assert_eq!(i_up.user_id, new_up.user_id);
        assert_eq!(i_up.name, new_up.name);
        assert_eq!(i_up.description, new_up.description);
        assert_eq!(i_up.color_id, new_up.color_id);
    }

    #[test]
    fn test_get_place() {
        let mut conn = establish_connection().unwrap();

        let user = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);

        let _p1 = get_user_place(
            &mut conn,
            Identifier::PlaceNameAndUserId(place.name, user.id),
        )
        .expect("No error");

        let _p2 = get_user_place(&mut conn, Identifier::UserId(user.id)).expect("No errror");
    }

    #[test]
    fn test_delete_place() {
        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);

        let deleted_places = delete_user_place(
            &mut conn,
            Identifier::PlaceNameAndUserId(place.name.clone(), user.id),
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

        let res = [
            delete_user_place(
                &mut conn,
                Identifier::PlaceNameAndUserId("namethatdoesntexist".to_string(), user.id),
            ),
            delete_user_place(
                &mut conn,
                Identifier::PlaceNameAndUserId(place.name, rand::random()),
            ),
        ];

        assert!(res.into_iter().all(|r| {
            if r.is_ok() {
                println!("r: {}", r.is_ok());
            }

            r.is_err_and(|e| match e {
                Error::NotFound(_) => true,
                _ => {
                    println!("was false, e: {e:?}");
                    false
                }
            })
        }))
    }
}
