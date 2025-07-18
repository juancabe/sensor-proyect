use crate::api::{ApiEndpoint, model::api_id::ApiId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RegisterRequestBody {
    pub username: String,
    pub hashed_password: String,
    pub email: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RegisterIncorrectReason {
    EmailUsed,
    UsernameUsed,
    HashInvalid,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RegisterResponseBody {
    Correct(ApiId),
    Incorrect(RegisterIncorrectReason),
}

pub struct Register {}

#[derive(Debug, Clone, Copy)]
pub enum RegisterResponseCode {
    Ok,
    BadRequest,
    PayloadTooLarge,
    Unauthorized,
    InternalServerError,
}

impl From<RegisterResponseCode> for http::StatusCode {
    fn from(code: RegisterResponseCode) -> Self {
        match code {
            RegisterResponseCode::Ok => http::StatusCode::OK,
            RegisterResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            RegisterResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            RegisterResponseCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            RegisterResponseCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for Register {
    type RequestBody = RegisterRequestBody;
    type ResponseBody = RegisterResponseBody;
    type ResponseCode = RegisterResponseCode;

    const PATH: &'static str = "/api/v0/register";
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
