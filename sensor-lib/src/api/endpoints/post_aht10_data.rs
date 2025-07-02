use crate::api::{ApiEndpoint, model::aht10_data::Aht10Data};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostAht10DataBody {
    pub user_uuid: String,
    pub user_place_id: i32,
    pub data: Aht10Data,
    pub added_at: Option<i64>,
}
pub struct PostAht10 {}

#[derive(Debug, Clone, Copy)]
pub enum PostAht10ResponseCode {
    Ok,
    BadRequest,
    PayloadTooLarge,
    InternalServerError,
}

impl From<PostAht10ResponseCode> for http::StatusCode {
    fn from(code: PostAht10ResponseCode) -> Self {
        match code {
            PostAht10ResponseCode::Ok => http::StatusCode::OK,
            PostAht10ResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            PostAht10ResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            PostAht10ResponseCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for PostAht10 {
    type RequestBody = PostAht10DataBody;
    type ResponseBody = ();
    type ResponseCode = PostAht10ResponseCode;

    const PATH: &'static str = "/api/v0/post_aht10_data";
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
