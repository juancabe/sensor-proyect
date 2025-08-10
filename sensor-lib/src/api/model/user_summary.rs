use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::api::model::{
    api_id::ApiId,
    color_palette::{PlaceColor, SensorColor},
    sensor_kind::SensorKind,
};

pub type Name = String;
pub type Description = String;
pub type UNIXTimestampSeconds = u32;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct SensorSummary {
    pub kind: SensorKind,
    pub api_id: ApiId,
    pub device_id: ApiId,
    pub last_update: u32,
    pub place_id: ApiId,
    pub name: Name,
    pub description: Option<Description>,
    pub color: SensorColor,
    pub last_serialized_data: Option<(String, UNIXTimestampSeconds)>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS, PartialEq, Eq, Hash)]
#[ts(export)]
pub struct PlaceSummary {
    pub place_id: ApiId,
    pub last_update: u32,
    pub name: Name,
    pub description: Option<Description>,
    pub color: PlaceColor,
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct UserSummary {
    pub username: String,
    pub email: String,
    pub sensors: Vec<SensorSummary>,
    pub places: Vec<PlaceSummary>,
}
