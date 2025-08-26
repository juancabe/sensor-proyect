use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

use crate::types::validate::{
    api_raw_password::ApiRawPassword, api_username::ApiUsername, device_id::DeviceId,
};

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/session/")]
pub struct UserLogin {
    #[validate]
    pub username: ApiUsername,
    #[validate]
    pub raw_password: ApiRawPassword,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/session/")]
pub struct SensorLogin {
    #[validate]
    pub device_id: DeviceId,
    #[validate(max_length = 128)]
    #[validate(min_length = 128)]
    pub random_message: String,
    #[validate(max_length = 128)]
    #[validate(min_length = 128)]
    #[validate(pattern = "^[0-9A-Fa-f]+$")] // Just HEX characters
    pub signature_of_message: String,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/session/")]
pub enum PostSession {
    User(#[validate] UserLogin),
    Sensor(#[validate] SensorLogin),
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/session/")]
pub struct PutSession {}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "./api/endpoints/session/")]
// WARN: Dont accept this in any endpoint
// WARN Every time this struct is returned, the response MUST return a Set-Cookie with the JWT
pub struct ApiSession {
    pub access_token: String,
    pub expires_in: usize,
    token_type: String,
}

impl ApiSession {
    pub fn new(access_token: String, expires_in: usize) -> Self {
        Self {
            access_token,
            expires_in,
            token_type: "Bearer".to_string(),
        }
    }
}
