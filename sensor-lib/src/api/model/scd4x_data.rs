use crate::api::model::SensorData;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

// Struct that holds the API data for the SCD4X sensor.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Scd4xData {
    pub sensor_id: String,
    // CO2 concentration in ppm
    pub co2: u16,
    // Humidity in percentage (0.0 to 100.0)
    pub humidity: f32,
    // Temperature in degrees Celsius
    pub temperature: f32,
}

impl Scd4xData {
    pub fn new(sensor_id: String, co2: u16, humidity: f32, temperature: f32) -> Self {
        Scd4xData {
            sensor_id,
            co2,
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

    pub fn get_co2(&self) -> u16 {
        self.co2
    }
}

impl SensorData for Scd4xData {
    fn get_sensor_id(&self) -> &str {
        self.get_sensor_id()
    }

    fn to_any_sensor_data(self) -> super::any_sensor_data::AnySensorData {
        super::any_sensor_data::AnySensorData::Scd4x(self)
    }
}
