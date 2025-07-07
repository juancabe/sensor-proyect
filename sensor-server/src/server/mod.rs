use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::{Method, Request, Response, StatusCode};
use sensor_lib::api::ApiEndpoint;
use sensor_lib::api::endpoints::get_aht10_data::GetAht10;
use sensor_lib::api::endpoints::post_sensor::{PostSensor, PostSensorResponseBody};
use sensor_lib::api::endpoints::post_sensor_data::{PostSensorData, PostSensorResponseCode};
use sensor_lib::api::model::sensor_kind::SensorKind;

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
            <title>Sensor Server</title>
        </head>
        <body>
            <h1>Welcome to the Sensor Server</h1>
        </body>
        </html>
        ";

            let mut response = Response::new(full(body));
            response
                .headers_mut()
                .append("content-type", HeaderValue::from_static("text/html"));
            Ok(response.into())
        }
        (&PostSensorData::METHOD, PostSensorData::PATH) => {
            let max_size = <PostSensorData as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;

            let (parsed_body, serialized_data) = match extract_body_and_parse(
                req,
                max_size,
                Some(PostSensorData::parse_request_body),
            )
            .await
            {
                Ok(post_body) => {
                    let serialized_data = match serde_json::to_string(&post_body.data) {
                        Ok(data) => data,
                        Err(err) => {
                            log::warn!("Failed to serialize AHT10 data: {}", err);
                            let mut response = Response::new(empty());
                            *response.status_mut() = StatusCode::BAD_REQUEST;
                            return Ok(response);
                        }
                    };
                    (post_body, serialized_data)
                }
                Err(ExtractError::PayloadTooLarge) => {
                    log::warn!("Request body too large");
                    let mut response = Response::new(empty());
                    *response.status_mut() = PostSensorResponseCode::PayloadTooLarge.into();
                    return Ok(response);
                }
                Err(ExtractError::ErrorReceiving) => {
                    log::error!("Error receiving request body");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsValue(err)) => {
                    log::warn!("Failed to parse body as JSON: {}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsType) => {
                    log::warn!("Failed to parse request body as type");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            };

            if !user_uuid_matches_sensor_api_id(&parsed_body.user_uuid, &parsed_body.sensor_api_id)
            {
                log::warn!(
                    "User UUID does not match sensor API ID: {} != {}",
                    parsed_body.user_uuid,
                    parsed_body.sensor_api_id
                );
                let mut response = Response::new(empty());
                *response.status_mut() = PostSensorResponseCode::Unauthorized.into();
                return Ok(response);
            }

            let e = match get_sensor_kind_from_id(&parsed_body.sensor_api_id) {
                Ok(kind) => match kind {
                    SensorKind::Aht10 => {
                        let data = models::NewAht10Data {
                            sensor: parsed_body.sensor_api_id,
                            serialized_data: &serialized_data,
                            added_at: parsed_body.added_at.unwrap_or_else(|| {
                                // Use current time in seconds if not provided
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .expect("Time went backwards")
                                    .as_secs() as i32
                            }),
                        };
                        save_new_aht10_data(data)
                    }
                    SensorKind::Scd4x => {
                        let data = models::NewScd4xData {
                            sensor: parsed_body.sensor_api_id,
                            serialized_data: &serialized_data,
                            added_at: parsed_body.added_at.unwrap_or_else(|| {
                                // Use current time in seconds if not provided
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .expect("Time went backwards")
                                    .as_secs() as i32
                            }),
                        };
                        save_new_scd4x_data(data)
                    }
                },
                Err(err) => {
                    log::error!("Error getting sensor kind: {:?}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            };

            match e {
                Err(e) => {
                    log::error!("Error saving new sensor data: {:?}", e);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
                Ok(()) => Ok(Response::new(empty())),
            }
        }
        (&GetAht10::METHOD, GetAht10::PATH) => {
            let max_size = <GetAht10 as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;

            let body =
                match extract_body_and_parse(req, max_size, Some(GetAht10::parse_request_body))
                    .await
                {
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
                    Err(ExtractError::ParseErrorAsValue(err)) => {
                        log::warn!("Failed to parse body as JSON: {}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsType) => {
                        log::warn!("Failed to parse request body as type");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                };

            let query_result = query_aht10_data(body);

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
        (&PostSensor::METHOD, PostSensor::PATH) => {
            let max_size = <PostSensor as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;
            // Extract the body from the request
            let body =
                match extract_body_and_parse(req, max_size, Some(PostSensor::parse_request_body))
                    .await
                {
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
                    Err(ExtractError::ParseErrorAsValue(err)) => {
                        log::warn!("Failed to parse body as JSON: {}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsType) => {
                        log::warn!("Failed to parse request body as type");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                };

            let user_uuid = &body.user_uuid;
            let user_place_id_ = body.user_place_id;
            let sensor_kind = body.sensor_kind;
            let ble_mac_address = &body.sensor_mac;

            match new_sensor(user_uuid, sensor_kind, user_place_id_, ble_mac_address) {
                Ok(user_sensor_api_id) => {
                    log::info!("New sensor created with API ID: {}", user_sensor_api_id);
                    let response_body = PostSensorResponseBody {
                        sensor_api_id: user_sensor_api_id.clone(),
                    };
                    let response_body_json = serde_json::to_string(&response_body)
                        .expect("Failed to serialize response body");

                    let mut response = Response::new(full(response_body_json));
                    response
                        .headers_mut()
                        .append("content-type", HeaderValue::from_static("application/json"));
                    return Ok(response);
                }
                Err(err) => {
                    log::error!("Error creating new sensor: {:?}", err);
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
