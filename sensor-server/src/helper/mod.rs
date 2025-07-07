use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{
    Request,
    body::{Body, Bytes},
};

#[derive(Debug)]
pub enum ExtractError {
    PayloadTooLarge,
    ErrorReceiving,
    ParseErrorAsValue(String),
    ParseErrorAsType,
}

async fn extract_body_as_bytes(
    req: Request<hyper::body::Incoming>,
    max_size: u64,
) -> Result<Bytes, ExtractError> {
    let body = req
        .collect()
        .await
        .map_err(|_| ExtractError::ErrorReceiving)?;

    let size = body.size_hint().upper().unwrap_or(0);
    if size > max_size {
        return Err(ExtractError::PayloadTooLarge);
    }

    Ok(body.to_bytes())
}
pub async fn extract_body_and_parse<T, E>(
    req: Request<hyper::body::Incoming>,
    max_size: u64,
    parse_request_body_fn: Option<fn(&serde_json::Value) -> Result<T, E>>,
) -> Result<T, ExtractError> {
    let body = extract_body_as_bytes(req, max_size).await?;
    if let Some(parse_fn) = parse_request_body_fn {
        let r = serde_json::from_slice::<serde_json::Value>(&body)
            .map_err(|err| {
                ExtractError::ParseErrorAsValue(format!("Failed to parse body as JSON: {}", err))
            })
            .and_then(|json_value| {
                parse_fn(&json_value).map_err(|_| ExtractError::ParseErrorAsType)
            })?;
        Ok(r)
    } else {
        Err(ExtractError::ErrorReceiving)
    }
}

// We create some utility functions to make Empty and Full bodies
// fit our broadened Response body type.
pub fn empty() -> BoxBody<Bytes, hyper::Error> {
    Empty::<Bytes>::new()
        .map_err(|never| match never {})
        .boxed()
}
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, hyper::Error> {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}
