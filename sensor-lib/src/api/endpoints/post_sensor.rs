use crate::api::{ApiEndpoint, model::sensor_kind::SensorKind};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostSensorRequestBody {
    pub user_api_id: String,
    pub user_place_id: i32,
    pub device_id: String,
    pub sensor_kind: SensorKind,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostSensorResponseBody {
    pub sensor_api_id: String,
}

pub struct PostSensor {}

#[derive(Debug, Clone, Copy)]
pub enum PostSensorResponseCode {
    Ok,
    BadRequest,
    PayloadTooLarge,
    Unauthorized,
    InternalServerError,
}

impl From<PostSensorResponseCode> for http::StatusCode {
    fn from(code: PostSensorResponseCode) -> Self {
        match code {
            PostSensorResponseCode::Ok => http::StatusCode::OK,
            PostSensorResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            PostSensorResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            PostSensorResponseCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            PostSensorResponseCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for PostSensor {
    type RequestBody = PostSensorRequestBody;
    type ResponseBody = PostSensorResponseBody;
    type ResponseCode = PostSensorResponseCode;

    const PATH: &'static str = "/api/v0/post_sensor";
    const METHOD: http::Method = http::Method::POST;

    const MAX_REQUEST_BODY_SIZE: u64 = 1024 * 2; // 2 KB
    const MAX_RESPONSE_BODY_SIZE: u64 = 1024 * 1024; // 1 MB

    fn parse_request_body(
        serde: &serde_json::Value,
    ) -> Result<Self::RequestBody, crate::api::BodyParseError> {
        serde_json::from_value(serde.clone())
            .map_err(|_| crate::api::BodyParseError::InvalidFormat("Invalid request body format"))
    }

    fn parse_response_body(
        serde: &serde_json::Value,
    ) -> Result<Self::ResponseBody, crate::api::BodyParseError> {
        serde
            .as_object()
            .and_then(|obj| obj.get("body"))
            .and_then(|body| serde_json::from_value(body.clone()).ok())
            .ok_or(crate::api::BodyParseError::InvalidFormat(
                "Invalid response body format",
            ))
    }
}
