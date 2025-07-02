use crate::api::{ApiEndpoint, model::aht10_data::Aht10Data};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetAht10RequestBody {
    pub user_uuid: String,
    pub user_place_id: i32,
    pub added_at_upper: Option<i64>,
    pub added_at_lower: Option<i64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetAht10ResponseBody {
    pub item_count: usize,
    pub data: Vec<Aht10Data>,
}

pub struct GetAht10 {}

#[derive(Debug, Clone, Copy)]
pub enum GetAht10ResponseCode {
    Ok,
    BadRequest,
    PayloadTooLarge,
    Unauthorized,
    InternalServerError,
}

impl From<GetAht10ResponseCode> for http::StatusCode {
    fn from(code: GetAht10ResponseCode) -> Self {
        match code {
            GetAht10ResponseCode::Ok => http::StatusCode::OK,
            GetAht10ResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            GetAht10ResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            GetAht10ResponseCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            GetAht10ResponseCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for GetAht10 {
    type RequestBody = GetAht10RequestBody;
    type ResponseBody = GetAht10ResponseBody;
    type ResponseCode = GetAht10ResponseCode;

    const PATH: &'static str = "/api/v0/get_aht10_data";
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
