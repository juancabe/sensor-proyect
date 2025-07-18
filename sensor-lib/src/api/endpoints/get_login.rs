use crate::api::{ApiEndpoint, model::api_id::ApiId};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetLoginRequestBody {
    pub username: String,
    pub hashed_password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GetLoginResponseBody {
    pub api_id: ApiId,
}

pub struct GetLogin {}

#[derive(Debug, Clone, Copy)]
pub enum GetLoginResponseCode {
    Ok,
    BadRequest,
    PayloadTooLarge,
    Unauthorized,
    InternalServerError,
}

impl From<GetLoginResponseCode> for http::StatusCode {
    fn from(code: GetLoginResponseCode) -> Self {
        match code {
            GetLoginResponseCode::Ok => http::StatusCode::OK,
            GetLoginResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            GetLoginResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            GetLoginResponseCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            GetLoginResponseCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for GetLogin {
    type RequestBody = GetLoginRequestBody;
    type ResponseBody = GetLoginResponseBody;
    type ResponseCode = GetLoginResponseCode;

    const PATH: &'static str = "/api/v0/get_login";
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
