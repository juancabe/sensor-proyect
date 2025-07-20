use crate::api::{ApiEndpoint, model::api_id::ApiId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export, export_to = "endpoints/Login.ts")]
pub struct LoginRequestBody {
    pub username: String,
    pub hashed_password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export, export_to = "endpoints/Login.ts")]
pub struct LoginResponseBody {
    pub api_id: ApiId,
}

pub struct Login {}

#[derive(Debug, Clone, Copy, TS)]
#[ts(export, export_to = "endpoints/Login.ts")]
pub enum LoginResponseCode {
    Ok,
    BadRequest,
    PayloadTooLarge,
    Unauthorized,
    InternalServerError,
}

impl From<LoginResponseCode> for http::StatusCode {
    fn from(code: LoginResponseCode) -> Self {
        match code {
            LoginResponseCode::Ok => http::StatusCode::OK,
            LoginResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            LoginResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            LoginResponseCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            LoginResponseCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for Login {
    type RequestBody = LoginRequestBody;
    type ResponseBody = LoginResponseBody;
    type ResponseCode = LoginResponseCode;

    const PATH: &'static str = "/api/v0/login";
    const METHOD: http::Method = http::Method::POST;

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
