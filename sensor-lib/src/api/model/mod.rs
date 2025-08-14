pub mod aht10_data;
pub mod any_sensor_data;
pub mod api_id;
pub mod color;
pub mod scd4x_data;
pub mod sensor_kind;
pub mod user_summary;

use serde::{Deserialize, Serialize};

use crate::api::model::any_sensor_data::AnySensorData;
pub trait SensorData: Serialize + for<'de> Deserialize<'de> {
    // Returns the sensor ID as a string slice
    fn get_sensor_id(&self) -> &str;

    // Grants polymorphism
    fn to_any_sensor_data(self) -> AnySensorData;
}
