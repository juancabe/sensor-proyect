use crate::api::{
    ApiEndpoint,
    model::{api_id::ApiId, color_palette::PlaceColor},
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export, export_to = "endpoints/PostPlace.ts")]
pub enum PostPlaceRequestBody {
    Create {
        username: String,
        user_api_id: ApiId,
        place_name: String,
        place_description: Option<String>,
        place_color: PlaceColor,
    },
    Delete {
        user_api_id: ApiId,
        place_id: ApiId,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone, TS)]
#[ts(export, export_to = "endpoints/PostPlace.ts")]
pub enum PostPlaceResponseBody {
    Created {
        place_id: ApiId,
        place_name: String,
        place_description: Option<String>,
    },
    NoContent,
}

pub struct PostPlace {}

#[derive(Debug, Clone, Copy, TS)]
#[ts(export, export_to = "endpoints/PostPlace.ts")]
pub enum PostPlaceResponseCode {
    Ok,
    BadRequest,
    PayloadTooLarge,
    Unauthorized,
    InternalServerError,
    NoContent,
}

impl From<PostPlaceResponseCode> for http::StatusCode {
    fn from(code: PostPlaceResponseCode) -> Self {
        match code {
            PostPlaceResponseCode::Ok => http::StatusCode::OK,
            PostPlaceResponseCode::BadRequest => http::StatusCode::BAD_REQUEST,
            PostPlaceResponseCode::PayloadTooLarge => http::StatusCode::PAYLOAD_TOO_LARGE,
            PostPlaceResponseCode::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            PostPlaceResponseCode::Unauthorized => http::StatusCode::UNAUTHORIZED,
            PostPlaceResponseCode::NoContent => http::StatusCode::NO_CONTENT,
        }
    }
}

impl<'a, 'b> ApiEndpoint<'a, 'b> for PostPlace {
    type RequestBody = PostPlaceRequestBody;
    type ResponseBody = PostPlaceResponseBody;
    type ResponseCode = PostPlaceResponseCode;

    const PATH: &'static str = "/api/v0/post_place";
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
