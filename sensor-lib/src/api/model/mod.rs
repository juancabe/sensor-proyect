pub mod aht10_data;
pub mod any_sensor_data;
pub mod scd4x_data;

use serde::{Deserialize, Serialize};

use crate::api::model::any_sensor_data::AnySensorData;
pub trait SensorData: Serialize + for<'de> Deserialize<'de> {
    // Returns the sensor ID as a string slice
    fn get_sensor_id(&self) -> &str;

    // Grants polymorphism
    fn to_any_sensor_data(self) -> AnySensorData;
}
