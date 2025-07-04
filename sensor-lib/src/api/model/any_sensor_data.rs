use serde::{Deserialize, Serialize};

use crate::api::model::{SensorData, aht10_data::Aht10Data, scd4x_data::Scd4xData};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "sensor_type")]
pub enum AnySensorData {
    Aht10(Aht10Data),
    Scd4x(Scd4xData),
}

impl SensorData for AnySensorData {
    fn get_sensor_id(&self) -> &str {
        match self {
            AnySensorData::Aht10(data) => data.get_sensor_id(),
            AnySensorData::Scd4x(data) => data.get_sensor_id(),
        }
    }

    fn to_any_sensor_data(self) -> AnySensorData {
        self
    }
}
