use common::{endpoints_io::sensor::SensorChange, types::validate::device_id::DeviceId};
use diesel::prelude::*;

use crate::{
    auth::sensor_claims::SensorClaims,
    db::{
        self, DbConn, Error, colors,
        model::{NewUserSensor, SensorData, UserPlace, UserSensor},
        user_places, users,
    },
};

#[derive(Debug, Clone)]
pub struct AuthorizedSensor(UserSensor);

impl AuthorizedSensor {
    pub fn from_username(
        conn: &mut DbConn,
        device_id: &DeviceId,
        username: &str,
    ) -> Result<Self, Error> {
        let (place, sensor) = _get_user_sensor_and_place_unauthorized(conn, device_id.as_str())?;

        let user_id = users::get_user(conn, users::Identifier::Username(username))?.id;

        if user_id != place.user_id {
            log::warn!(
                "User ({}) tried to operate with sensor ({}) that didn't belong to him",
                username,
                device_id.as_str()
            );
            Err(Error::NotFound("Sensor not found".into()))
        } else {
            Ok(Self(sensor))
        }
    }

    pub fn from_access_id(
        conn: &mut DbConn,
        device_id: &DeviceId,
        access_id: &DeviceId,
    ) -> Result<Self, Error> {
        let (_, sensor) = _get_user_sensor_and_place_unauthorized(conn, device_id.as_str())?;
        if sensor.access_id.as_str() != access_id.as_str() {
            log::warn!(
                "Access Id ({}) tried to operate with sensor ({}) that didn't match",
                access_id.as_str(),
                device_id.as_str()
            );
            Err(Error::NotFound("Sensor not found".into()))
        } else {
            Ok(Self(sensor))
        }
    }

    pub fn from_sensor_claims(conn: &mut DbConn, claims: &SensorClaims) -> Result<Self, Error> {
        let (_, sensor) = _get_user_sensor_and_place_unauthorized(conn, claims.device_id.as_str())?;
        Ok(Self(sensor))
    }

    pub fn get(self) -> UserSensor {
        self.0
    }
}

pub fn insert_user_sensor(conn: &mut DbConn, sensor: NewUserSensor) -> Result<UserSensor, Error> {
    use crate::db::schema::user_sensors::dsl::user_sensors as user_sensors_table;

    let sensor: Vec<UserSensor> = sensor.insert_into(user_sensors_table).load(conn)?;

    sensor
        .first()
        .ok_or(Error::NotFound("The returned vec was empty".into()))
        .map(|e| e.clone())
}

fn _get_user_sensor_and_place_unauthorized(
    conn: &mut DbConn,
    device_id: &str,
) -> Result<(UserPlace, UserSensor), Error> {
    use crate::db::schema::{
        user_sensors::dsl as user_sensor, user_sensors::dsl::user_sensors as user_sensors_table,
    };

    use crate::db::schema::user_places::dsl::user_places as user_places_table;

    let res = user_sensors_table
        .filter(user_sensor::device_id.eq(device_id))
        .inner_join(user_places_table)
        .select((
            db::model::UserPlace::as_select(),
            db::model::UserSensor::as_select(),
        ))
        .load(conn)?
        .into_iter()
        .next()
        .ok_or_else(|| Error::NotFound("Sensor not found".into()))?;

    Ok(res)
}

pub enum Identifier<'a> {
    PlaceNameAndUserId(&'a str, i32),
    SensorDeviceId(AuthorizedSensor),
}

pub fn get_user_sensor_and_place_and_last_data(
    conn: &mut DbConn,
    identifier: Identifier,
) -> Result<Vec<(UserPlace, UserSensor, Option<SensorData>)>, Error> {
    let res = match identifier {
        Identifier::SensorDeviceId(auth_sensor) => {
            use crate::db::schema::user_places::dsl::user_places as user_places_table;
            use crate::db::schema::{
                user_sensors::dsl as user_sensor,
                user_sensors::dsl::user_sensors as user_sensors_table,
            };

            let sensor = auth_sensor.get();

            let res: Vec<(UserPlace, UserSensor)> = user_sensors_table
                .filter(user_sensor::device_id.eq(sensor.device_id))
                .inner_join(user_places_table)
                .select((
                    db::model::UserPlace::as_select(),
                    db::model::UserSensor::as_select(),
                ))
                .load(conn)?;

            res
        }
        Identifier::PlaceNameAndUserId(name, user_id) => {
            use crate::db::schema::user_sensors::dsl::user_sensors as user_sensors_table;
            use crate::db::schema::{
                user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
            };

            let res = user_places_table
                .filter(user_place::name.eq(name))
                .filter(user_place::user_id.eq(user_id))
                .inner_join(user_sensors_table)
                .select((
                    db::model::UserPlace::as_select(),
                    db::model::UserSensor::as_select(),
                ))
                .load(conn)?;

            res
        }
    };

    use crate::db::schema::{
        sensor_data::dsl as sensor_datum, sensor_data::dsl::sensor_data as sensor_data_table,
    };

    let mut resp = vec![];
    for (place, sensor) in res {
        let data = sensor_data_table
            .filter(sensor_datum::sensor_id.eq(sensor.id))
            .order(sensor_datum::added_at.desc())
            .limit(1)
            .load::<SensorData>(conn)?
            .into_iter()
            .next();

        resp.push((place, sensor, data));
    }
    Ok(resp)
}

pub type Update = SensorChange;

pub fn update_user_sensor(
    conn: &mut DbConn,
    auth_sensor: AuthorizedSensor,
    update: Update,
    user_id: i32,
) -> Result<UserSensor, Error> {
    use crate::db::schema::{
        user_sensors::dsl as user_sensor, user_sensors::dsl::user_sensors as user_sensors_table,
    };

    let mut sensor = auth_sensor.get();

    match update {
        SensorChange::PlaceName(api_entity_name) => {
            sensor.place_id = user_places::get_user_place(
                conn,
                user_places::Identifier::PlaceNameAndUserId(api_entity_name.as_str(), user_id),
            )?
            .into_iter()
            .next()
            .ok_or_else(|| Error::NotFound("Place not found".into()))?
            .id
        }
        SensorChange::Name(api_entity_name) => sensor.name = api_entity_name.into(),
        SensorChange::Description(api_description) => {
            sensor.description = api_description.map(|d| d.into())
        }
        SensorChange::Color(api_color) => {
            sensor.color_id = colors::get_color_id(conn, colors::Identifier::Hex(api_color.into()))?
        }
    }

    let rows = diesel::update(user_sensors_table)
        .filter(user_sensor::id.eq(sensor.id))
        .set(&sensor)
        .execute(conn)?;

    if rows == 0 {
        Err(Error::NotFound(
            "Not found, update didn't affect any rows".into(),
        ))?
    }

    Ok(sensor)
}

pub fn delete_user_sensor(
    conn: &mut DbConn,
    identifier: Identifier,
) -> Result<Vec<(UserPlace, UserSensor)>, Error> {
    match identifier {
        Identifier::SensorDeviceId(auth_sensor) => {
            use crate::db::schema::{
                user_places::dsl as user_place, user_places::dsl::user_places as user_places_table,
            };
            use crate::db::schema::{
                user_sensors::dsl as user_sensor,
                user_sensors::dsl::user_sensors as user_sensors_table,
            };

            let sensor = auth_sensor.get();

            let sensor = user_sensors_table
                .filter(user_sensor::device_id.eq(sensor.device_id))
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
            use crate::db::schema::{
                user_sensors::dsl as user_sensor,
                user_sensors::dsl::user_sensors as user_sensors_table,
            };

            use crate::db::schema::{
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
        db::model::NewUserSensor,
        db::{
            establish_connection,
            tests::{create_test_user, create_test_user_place, create_test_user_sensor},
        },
    };

    use super::*;

    #[test]
    fn test_new_sensor() {
        let mut conn = establish_connection(true).unwrap();
        let (user, _) = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);

        let new_up: NewUserSensor = NewUserSensor {
            name: "new_testuserplace".to_string(),
            description: Some("le description".to_string()),
            color_id: 1,
            place_id: place.id,
            device_id: DeviceId::random().to_string(),
            access_id: DeviceId::random().to_string(),
        };

        let i_up = insert_user_sensor(&mut conn, new_up.clone()).expect("No errors expected");

        assert_eq!(i_up.name, new_up.name);
        assert_eq!(i_up.description, new_up.description);
        assert_eq!(i_up.color_id, new_up.color_id);
        assert_eq!(i_up.place_id, new_up.place_id);
    }

    #[test]
    fn test_get_sensor() {
        let mut conn = establish_connection(true).unwrap();

        let (user, _) = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);

        let _p1 = get_user_sensor_and_place_and_last_data(
            &mut conn,
            Identifier::PlaceNameAndUserId(&place.name, user.id),
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
        let mut conn = establish_connection(true).unwrap();
        let (user, _) = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);
        let user_sensor = create_test_user_sensor(&mut conn, &user_place);

        let deleted_sensors = delete_user_sensor(
            &mut conn,
            Identifier::PlaceNameAndUserId(&user_place.name, user.id),
        )
        .expect("Delete operation should not fail");

        assert_eq!(deleted_sensors.len(), 1);
        assert_eq!(deleted_sensors[0].1.id, user_sensor.id);
        // assert_eq!(deleted_sensors[0].name, place.name);

        use crate::db::schema::{
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
    }
}
