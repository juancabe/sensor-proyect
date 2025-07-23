use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(PartialEq, Eq, Deserialize, Serialize, Debug, Clone, Copy, TS)]
#[ts(export)]
pub enum SensorKind {
    Aht10 = 1,
    Scd4x = 2,
}

impl SensorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SensorKind::Aht10 => "aht10",
            SensorKind::Scd4x => "scd4x",
        }
    }

    pub fn from_str(s: &str) -> Option<SensorKind> {
        match s {
            "aht10" => Some(SensorKind::Aht10),
            "scd4x" => Some(SensorKind::Scd4x),
            _ => None,
        }
    }

    pub fn from_u8(value: u8) -> Option<SensorKind> {
        match value {
            1 => Some(SensorKind::Aht10),
            2 => Some(SensorKind::Scd4x),
            _ => None,
        }
    }

    pub fn from_i32(value: i32) -> Option<SensorKind> {
        match value {
            1 => Some(SensorKind::Aht10),
            2 => Some(SensorKind::Scd4x),
            _ => None,
        }
    }
}
