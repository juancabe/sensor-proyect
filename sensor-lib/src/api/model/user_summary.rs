use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::api::model::{api_id::ApiId, sensor_kind::SensorKind};

pub type Name = String;
pub type Description = String;
pub type PlaceId = u32;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
pub struct SensorSummary {
    pub kind: SensorKind,
    pub api_id: ApiId,
    pub device_id: ApiId,
    pub last_update: u32,
    pub place: PlaceId,
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export)]
pub struct UserSummary {
    pub username: String,
    pub email: String,
    pub sensors: Vec<SensorSummary>,
    pub places: Vec<(PlaceId, Name, Option<Description>)>,
}
