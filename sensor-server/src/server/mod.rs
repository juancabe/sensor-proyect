use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::{Method, Request, Response, StatusCode};
use sensor_lib::api::ApiEndpoint;
use sensor_lib::api::endpoints::get_aht10_data::GetAht10;
use sensor_lib::api::endpoints::post_aht10_data::{PostAht10, PostAht10ResponseCode};

use crate::db::*;
use crate::{helper::*, models};

pub async fn server(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => {
            let body = "<!DOCTYPE html>
        <html>
        <head>
            <title>Echo Server</title>
        </head>
        <body>
            <h1>Welcome to the Echo Server</h1>
            <p>Try sending a POST request to <code>/echo</code> or <code>/echo/uppercase</code> to see how it works.</p>
        </body>
        </html>
        ";

            let mut response = Response::new(full(body));
            response
                .headers_mut()
                .append("content-type", HeaderValue::from_static("text/html"));
            Ok(response.into())
        }

        // For post_aht10_data
        (&PostAht10::METHOD, PostAht10::PATH) => {
            let max_size = <PostAht10 as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;
            // Extract the body from the request
            let body = match extract_body_as_bytes(req, max_size).await {
                Ok(bytes) => bytes,
                Err(ExtractError::PayloadTooLarge) => {
                    log::warn!("Request body too large");
                    let mut response = Response::new(empty());
                    *response.status_mut() = PostAht10ResponseCode::PayloadTooLarge.into();
                    return Ok(response);
                }
                Err(ExtractError::ErrorReceiving) => {
                    log::error!("Error receiving request body");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            };

            let parse_result = {
                log::info!("Parsing request body: {:?}", body);

                serde_json::from_slice::<serde_json::Value>(&body)
                    .map_err(|err| format!("Failed to parse body as JSON: {}", err))
                    .and_then(|value| {
                        PostAht10::parse_request_body(&value)
                            .map_err(|err| format!("Failed to parse request body: {:?}", err))
                    })
                    .and_then(|post_body| {
                        let sd = serde_json::to_string(&post_body.data)
                            .map_err(|err| format!("Failed to serialize AHT10 data: {:?}", err))?;
                        Ok((post_body, sd))
                    })
            };

            let (parsed_body, serialized_data) = match parse_result {
                Ok(value) => value,
                Err(err) => {
                    log::warn!("PARSING: {}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = PostAht10ResponseCode::BadRequest.into();
                    return Ok(response);
                }
            };

            let new_data = models::NewAht10Data {
                user_uuid: &parsed_body.user_uuid,
                user_place_id: parsed_body.user_place_id,
                serialized_data: &serialized_data,
                added_at: parsed_body.added_at.unwrap_or_else(|| {
                    // Use current time in seconds if not provided
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs() as i64
                }),
            };

            if let Err(err) = save_new_aht10_data(new_data) {
                log::error!("Error saving new AHT10 data: {:?}", err);
                let mut response = Response::new(empty());
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                return Ok(response);
            }

            Ok(Response::new(empty()))
        }

        (&GetAht10::METHOD, GetAht10::PATH) => {
            let max_size = <GetAht10 as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;
            // Extract the body from the request
            let body = match extract_body_as_bytes(req, max_size).await {
                Ok(bytes) => bytes,
                Err(ExtractError::PayloadTooLarge) => {
                    log::warn!("Request body too large");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
                    return Ok(response);
                }
                Err(ExtractError::ErrorReceiving) => {
                    log::error!("Error receiving request body");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            };

            let parse_result = {
                log::info!("Parsing request body: {:?}", body);

                serde_json::from_slice::<serde_json::Value>(&body)
                    .map_err(|err| format!("Failed to parse body as JSON: {}", err))
                    .and_then(|value| {
                        GetAht10::parse_request_body(&value)
                            .map_err(|err| format!("Failed to parse request body: {:?}", err))
                    })
            };

            let parsed_body = match parse_result {
                Ok(value) => value,
                Err(err) => {
                    log::warn!("PARSING: {}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            };

            let query_result = query_aht10_data(parsed_body);

            let query_string = match query_result {
                Ok(data) => serde_json::to_string(&data),
                Err(err) => {
                    log::error!("Error querying AHT10 data: {:?}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            };

            match query_string {
                Ok(json_data) => {
                    let mut response = Response::new(full(json_data));
                    response
                        .headers_mut()
                        .append("content-type", HeaderValue::from_static("application/json"));
                    return Ok(response);
                }
                Err(err) => {
                    log::error!("Error serializing AHT10 data: {:?}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            }
        }

        // Return 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
