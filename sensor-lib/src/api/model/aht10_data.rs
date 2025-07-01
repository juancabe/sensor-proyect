use crate::api::model::SensorData;
use serde::{Deserialize, Serialize};

// Struct that holds the API data for the AHT10 sensor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aht10Data {
    pub sensor_id: String,
    // Humidity in percentage (0.0 to 100.0)
    pub humidity: f32,
    // Temperature in degrees Celsius
    pub temperature: f32,
}

impl Aht10Data {
    pub fn new(sensor_id: String, humidity: f32, temperature: f32) -> Self {
        Aht10Data {
            sensor_id,
            humidity,
            temperature,
        }
    }

    pub fn get_sensor_id(&self) -> &str {
        &self.sensor_id
    }

    pub fn get_humidity(&self) -> f32 {
        self.humidity
    }

    pub fn get_temperature(&self) -> f32 {
        self.temperature
    }
}

impl SensorData for Aht10Data {
    fn get_sensor_id(&self) -> &str {
        self.get_sensor_id()
    }
}
