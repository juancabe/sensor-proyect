use hyper::StatusCode;

use crate::{
    api::{Endpoint, types::api_id::ApiId},
    db::{self, DbConn, Error},
    model::UserSensor,
};

pub mod health;
pub mod place;
pub mod sensor;
pub mod sensor_data;
pub mod session;
pub mod user;

pub fn generate_endpoints() -> Vec<Box<dyn Endpoint>> {
    let mut endpoints = Vec::<Box<dyn Endpoint>>::new();

    endpoints.push(Box::new(place::Place::new()));
    endpoints.push(Box::new(sensor::Sensor::new()));
    endpoints.push(Box::new(sensor_data::SensorData::new()));
    endpoints.push(Box::new(session::Session::new()));
    endpoints.push(Box::new(user::User::new()));
    endpoints.push(Box::new(health::Health::new()));

    endpoints
}

fn authorized_sensor(
    conn: &mut DbConn,
    device_id: &ApiId,
    username: &str,
) -> Result<UserSensor, StatusCode> {
    let db_res = db::user_sensors::get_user_sensor(
        conn,
        db::user_sensors::Identifier::SensorDeviceId(device_id),
    )?;

    let (place, sensor) = db_res
        .into_iter()
        .next()
        .ok_or(Error::NotFound("NotFound".into()))?;

    let user_id = db::users::get_user(conn, db::users::Identifier::Username(username))?.id;

    if user_id != place.user_id {
        log::warn!(
            "User ({}) tried to operate with sensor ({}) that didn't belong to him",
            username,
            device_id.as_str()
        );
        Err(StatusCode::UNAUTHORIZED)
    } else {
        Ok(sensor)
    }
}
