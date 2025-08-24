use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

use crate::{
    endpoints_io::session::ApiSession,
    types::{ApiTimestamp, validate::device_id::DeviceId},
};

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor_data/")]
pub struct ApiSensorData {
    #[validate(max_length = 500)]
    pub data: String,
    pub added_at: ApiTimestamp,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor_data/")]
// WARN: Dont accept this in any endpoint
pub struct PostSensorDataResponse {
    pub api_data: ApiSensorData,
    pub new_session: ApiSession,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor_data/")]
pub struct GetSensorData {
    pub device_id: DeviceId,
    // Not included if added_at == [upper | lowest]_added_at
    pub lowest_added_at: Option<ApiTimestamp>,
    pub upper_added_at: Option<ApiTimestamp>,
}

#[derive(TS, Clone, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor_data/")]
pub struct PostSensorData {
    #[validate(max_length = 500)]
    pub serialized_data: String,
    pub created_at: Option<ApiTimestamp>,
}
