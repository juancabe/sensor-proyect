use crate::api::{
    ApiEndpoint,
    model::{any_sensor_data::AnySensorData, api_id::ApiId},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export, export_to = "endpoints/PostSensorData.ts")]
pub struct PostSensorDataRequestBody {
    pub user_api_id: ApiId,
    pub sensor_api_id: ApiId,
    pub data: AnySensorData,
    pub added_at: Option<u32>, // UNIX timestamp seconds
}
pub struct PostSensorData {}

#[derive(Debug, Clone, Copy, TS)]
#[ts(export, export_to = "endpoints/PostSensorData.ts")]
pub enum PostSensorResponseCode {
    Ok,
    BadRequest,
    Unauthorized,
    PayloadTooLarge,
    InternalServerError,
}

impl PostSensorResponseCode {
    pub fn from_u16(status: u16) -> Result<Self, u16> {
        let status = http::StatusCode::from_u16(status).map_err(|_| status)?;
        match status {
            http::StatusCode::OK => Ok(PostSensorResponseCode::Ok),
            http::StatusCode::BAD_REQUEST => Ok(PostSensorResponseCode::BadRequest),
            http::StatusCode::UNAUTHORIZED => Ok(PostSensorResponseCode::Unauthorized),
            http::StatusCode::PAYLOAD_TOO_LARGE => Ok(PostSensorResponseCode::PayloadTooLarge),
            http::StatusCode::INTERNAL_SERVER_ERROR => {
                Ok(PostSensorResponseCode::InternalServerError)
            }
            _ => Err(status.as_u16()),
        }
    }
}

impl From<PostSensorResponseCode> for http::StatusCode {
    fn from(code: PostSensorResponseCode) -> Self {
        match code {
            PostSensorResponseCode::Ok => http::StatusCode::OK,
            PostSensorResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            PostSensorResponseCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
            PostSensorResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            PostSensorResponseCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for PostSensorData {
    type RequestBody = PostSensorDataRequestBody;
    type ResponseBody = ();
    type ResponseCode = PostSensorResponseCode;

    const PATH: &'static str = "/api/v0/post_sensor_data";
    const METHOD: http::Method = http::Method::POST;

    const MAX_REQUEST_BODY_SIZE: u64 = 1024; // 1 KB
    const MAX_RESPONSE_BODY_SIZE: u64 = 1024; // 1 KB

    fn parse_request_body(
        serde: &serde_json::Value,
    ) -> Result<Self::RequestBody, crate::api::BodyParseError> {
        serde_json::from_value(serde.clone())
            .map_err(|_| crate::api::BodyParseError::InvalidFormat("Invalid request body format"))
    }

    fn parse_response_body(
        _serde: &serde_json::Value,
    ) -> Result<Self::ResponseBody, crate::api::BodyParseError> {
        Ok(())
    }
}
