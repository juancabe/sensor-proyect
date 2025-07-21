use crate::api::{
    ApiEndpoint,
    model::{api_id::ApiId, user_summary::UserSummary},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export, export_to = "endpoints/PostUserSummary.ts")]
pub struct PostUserSummaryRequestBody {
    pub username: String,
    pub user_api_id: ApiId,
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export, export_to = "endpoints/PostUserSummary.ts")]
pub struct PostUserSummaryResponseBody {
    pub summary: UserSummary,
}

pub struct PostUserSummary {}

#[derive(Debug, Clone, Copy, TS)]
#[ts(export, export_to = "endpoints/PostUserSummary.ts")]
pub enum PostUserSummaryResponseCode {
    Ok,
    BadRequest,
    PayloadTooLarge,
    Unauthorized,
    InternalServerError,
}

impl From<PostUserSummaryResponseCode> for http::StatusCode {
    fn from(code: PostUserSummaryResponseCode) -> Self {
        match code {
            PostUserSummaryResponseCode::Ok => http::StatusCode::OK,
            PostUserSummaryResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            PostUserSummaryResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            PostUserSummaryResponseCode::InternalServerError => {
                http::StatusCode::INTERNAL_SERVER_ERROR
            }
            PostUserSummaryResponseCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for PostUserSummary {
    type RequestBody = PostUserSummaryRequestBody;
    type ResponseBody = PostUserSummaryResponseBody;
    type ResponseCode = PostUserSummaryResponseCode;

    const PATH: &'static str = "/api/v0/post_user_summary";
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
