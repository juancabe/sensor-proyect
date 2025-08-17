use crate::api::Endpoint;

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
