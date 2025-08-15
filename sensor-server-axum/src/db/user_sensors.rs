use diesel::prelude::*;

use crate::{
    api::types::api_id::ApiId,
    db::{DbConn, Error},
    model::{self, NewUserSensor, UserPlace, UserSensor},
};

pub fn insert_user_sensor(conn: &mut DbConn, sensor: NewUserSensor) -> Result<UserSensor, Error> {
    use crate::schema::user_sensors::dsl::user_sensors as user_sensors_table;

    let sensor: Vec<UserSensor> = sensor.insert_into(user_sensors_table).load(conn)?;

    sensor
        .first()
        .ok_or(Error::NotFound("The returned vec was empty".into()))
        .map(|e| e.clone())
}

pub enum Identifier {
    PlaceNameAndUserId(String, i32),
    SensorDeviceId(ApiId),
}

pub fn get_user_sensor(
    conn: &mut DbConn,
    identifier: Identifier,
) -> Result<Vec<(UserPlace, UserSensor)>, Error> {
    match identifier {
        Identifier::SensorDeviceId(device_id) => {
            use crate::schema::user_places::dsl::user_places as user_places_table;
            use crate::schema::{
                user_sensors::dsl as user_sensor,
                user_sensors::dsl::user_sensors as user_sensors_table,
            };

            let res = user_sensors_table
                .filter(user_sensor::device_id.eq(device_id.as_str()))
                .inner_join(user_places_table)
                .select((
                    model::UserPlace::as_select(),
                    model::UserSensor::as_select(),
                ))
                .load(conn)?;

            Ok(res)
        }
        Identifier::PlaceNameAndUserId(name, user_id) => {
            use crate::schema::user_sensors::dsl::user_sensors as user_sensors_table;
            use crate::schema::{
                user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
            };

            let res = user_places_table
                .filter(user_place::name.eq(name))
                .filter(user_place::user_id.eq(user_id))
                .inner_join(user_sensors_table)
                .select((
                    model::UserPlace::as_select(),
                    model::UserSensor::as_select(),
                ))
                .load(conn)?;

            Ok(res)
        }
    }
}

pub fn delete_user_sensor(
    conn: &mut DbConn,
    identifier: Identifier,
) -> Result<Vec<(UserPlace, UserSensor)>, Error> {
    match identifier {
        Identifier::SensorDeviceId(device_id) => {
            use crate::schema::{
                user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
            };
            use crate::schema::{
                user_sensors::dsl as user_sensor,
                user_sensors::dsl::user_sensors as user_sensors_table,
            };

            let sensor = user_sensors_table
                .filter(user_sensor::device_id.eq(device_id.as_str()))
                .first::<UserSensor>(conn)?;

            let place = user_places_table
                .filter(user_place::id.eq(sensor.place_id))
                .first::<UserPlace>(conn)?;

            let deleted_sensors =
                diesel::delete(user_sensors_table.filter(user_sensor::id.eq(sensor.id)))
                    .get_results(conn)?
                    .into_iter()
                    .map(|sensor| (place.clone(), sensor));

            Ok(deleted_sensors.collect())
        }
        Identifier::PlaceNameAndUserId(name, user_id) => {
            use crate::schema::{
                user_sensors::dsl as user_sensor,
                user_sensors::dsl::user_sensors as user_sensors_table,
            };

            use crate::schema::{
                user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
            };

            let place = user_places_table
                .filter(user_place::name.eq(name))
                .filter(user_place::user_id.eq(user_id))
                .first::<UserPlace>(conn)?;

            let deleted_sensors =
                diesel::delete(user_sensors_table.filter(user_sensor::place_id.eq(place.id)))
                    .get_results::<UserSensor>(conn)?;

            if deleted_sensors.len() < 1 {
                Err(Error::NotFound("No sensors deleted by sensor id".into()))
            } else {
                Ok(deleted_sensors
                    .into_iter()
                    .map(|sensor| (place.clone(), sensor))
                    .collect())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        db::{
            establish_connection,
            tests::{create_test_user, create_test_user_place, create_test_user_sensor},
        },
        model::NewUserSensor,
    };

    use super::*;

    #[test]
    fn test_new_sensor() {
        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);

        let new_up: NewUserSensor = NewUserSensor {
            name: "new_testuserplace".to_string(),
            description: Some("le description".to_string()),
            color_id: 1,
            place_id: place.id,
            device_id: ApiId::random().to_string(),
        };

        let i_up = insert_user_sensor(&mut conn, new_up.clone()).expect("No errors expected");

        assert_eq!(i_up.name, new_up.name);
        assert_eq!(i_up.description, new_up.description);
        assert_eq!(i_up.color_id, new_up.color_id);
        assert_eq!(i_up.place_id, new_up.place_id);
    }

    #[test]
    fn test_get_sensor() {
        let mut conn = establish_connection().unwrap();

        let user = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);

        let _p1 = get_user_sensor(
            &mut conn,
            Identifier::PlaceNameAndUserId(place.name.clone(), user.id),
        )
        .expect("No error");

        // let _p2 = get_user_sensor(
        //     &mut conn,
        //     Identifier::SensorApiId(ApiId::from_string(&user.api_id).expect("ApiId valid")),
        // )
        // .expect("No errror");
    }

    #[test]
    fn test_delete_sensor() {
        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);
        let user_sensor = create_test_user_sensor(&mut conn, &user_place);

        let deleted_sensors = delete_user_sensor(
            &mut conn,
            Identifier::PlaceNameAndUserId(user_place.name.clone(), user.id),
        )
        .expect("Delete operation should not fail");

        assert_eq!(deleted_sensors.len(), 1);
        assert_eq!(deleted_sensors[0].1.id, user_sensor.id);
        // assert_eq!(deleted_sensors[0].name, place.name);

        use crate::schema::{
            user_sensors::dsl as user_sensor, user_sensors::dsl::user_sensors as user_sensors_table,
        };

        let find_result = user_sensors_table
            .filter(user_sensor::place_id.eq(user_place.id))
            .select(user_sensor::id) // We only need to know if it exists, so just select the id
            .first::<i32>(&mut conn)
            .optional(); // .optional() makes it return Ok(None) instead of Err(NotFound)

        assert_eq!(
            find_result.expect("DB query should not fail"),
            None,
            "The place with id {} should not be found after deletion.",
            user_place.id
        );

        // let res = [
        //     delete_user_sensor(&mut conn, Identifier::UserApiId(ApiId::random())),
        //     delete_user_sensor(&mut conn, Identifier::PlaceApiId(ApiId::random())),
        // ];
        //
        // assert!(res.into_iter().all(|r| {
        //     if r.is_ok() {
        //         println!("r: {}", r.is_ok());
        //     }
        //
        //     r.is_err_and(|e| match e {
        //         Error::NotFound(_) => true,
        //         _ => {
        //             println!("was false, e: {e:?}");
        //             false
        //         }
        //     })
        // }))
    }
}
