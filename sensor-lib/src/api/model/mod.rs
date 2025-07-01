pub mod aht10_data;

use serde::{Deserialize, Serialize};

pub trait SensorData: Serialize + for<'de> Deserialize<'de> {
    // Returns the sensor ID as a string slice
    fn get_sensor_id(&self) -> &str;
}
