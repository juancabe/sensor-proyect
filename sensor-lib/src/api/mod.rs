use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum BodyParseError {
    InvalidFormat(&'static str),
}

pub trait ApiEndpoint<'a, 'b>
where
    Self::ResponseCode: Into<http::StatusCode>,
{
    type RequestBody: Serialize + Deserialize<'a>; // Type for the request body
    type ResponseBody: Serialize + Deserialize<'a>; // Type for the response body
    type ResponseCode: Copy;

    const PATH: &'static str;
    const METHOD: http::Method;

    const MAX_REQUEST_BODY_SIZE: u64;
    const MAX_RESPONSE_BODY_SIZE: u64;

    fn parse_request_body(serde: &serde_json::Value) -> Result<Self::RequestBody, BodyParseError>;
    fn parse_response_body(serde: &serde_json::Value)
    -> Result<Self::ResponseBody, BodyParseError>;
}

pub mod endpoints;
pub mod model;
