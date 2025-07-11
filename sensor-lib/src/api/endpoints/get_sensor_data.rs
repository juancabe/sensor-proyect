use crate::api::{ApiEndpoint, model::aht10_data::Aht10Data};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetSensorRequestBody {
    pub user_api_id: String,
    pub sensor_api_id: String,
    pub added_at_upper: Option<NaiveDateTime>,
    pub added_at_lower: Option<NaiveDateTime>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetSensorResponseBody {
    pub item_count: usize,
    pub serialized_data: Vec<String>,
}

pub struct GetSensor {}

#[derive(Debug, Clone, Copy)]
pub enum GetSensorResponseCode {
    Ok,
    BadRequest,
    PayloadTooLarge,
    Unauthorized,
    InternalServerError,
}

impl From<GetSensorResponseCode> for http::StatusCode {
    fn from(code: GetSensorResponseCode) -> Self {
        match code {
            GetSensorResponseCode::Ok => http::StatusCode::OK,
            GetSensorResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            GetSensorResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            GetSensorResponseCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            GetSensorResponseCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for GetSensor {
    type RequestBody = GetSensorRequestBody;
    type ResponseBody = GetSensorResponseBody;
    type ResponseCode = GetSensorResponseCode;

    const PATH: &'static str = "/api/v0/get_sensor_data";
    const METHOD: http::Method = http::Method::GET;

    const MAX_REQUEST_BODY_SIZE: u64 = 1024; // 1 KB
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
