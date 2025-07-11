use diesel::query_dsl::methods::SelectDsl;
use diesel::{RunQueryDsl, SelectableHelper};
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::{Method, Request, Response, StatusCode};
use sensor_lib::api::ApiEndpoint;
use sensor_lib::api::endpoints::get_login::GetLogin;
use sensor_lib::api::endpoints::get_sensor_data::GetSensor;
use sensor_lib::api::endpoints::post_sensor::{PostSensor, PostSensorResponseBody};
use sensor_lib::api::endpoints::post_sensor_data::{PostSensorData, PostSensorResponseCode};
use sensor_lib::api::model::sensor_kind::SensorKind;

use crate::db;
use crate::{helper::*, models};

pub async fn server(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::info!(
        "Serving method: {}, at path: {}",
        req.method(),
        req.uri().path()
    );

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

            log::info!("[/] Returning 200 Status Code");
            let mut response = Response::new(full(body));
            response
                .headers_mut()
                .append("content-type", HeaderValue::from_static("text/html"));
            Ok(response.into())
        }
        (&PostSensorData::METHOD, PostSensorData::PATH) => {
            log::info!("[PostSensorData] Request matched PostSensorData");

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
                            log::warn!("[PostSensorData] Failed to serialize Sensor data: {}", err);
                            let mut response = Response::new(empty());
                            *response.status_mut() = StatusCode::BAD_REQUEST;
                            return Ok(response);
                        }
                    };
                    (post_body, serialized_data)
                }
                Err(ExtractError::PayloadTooLarge) => {
                    log::warn!("[PostSensorData] Request body too large");
                    let mut response = Response::new(empty());
                    *response.status_mut() = PostSensorResponseCode::PayloadTooLarge.into();
                    return Ok(response);
                }
                Err(ExtractError::ErrorReceiving) => {
                    log::error!("[PostSensorData] Error receiving request body");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsValue(err)) => {
                    log::warn!("[PostSensorData] Failed to parse body as JSON: {}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsType) => {
                    log::warn!("[PostSensorData] Failed to parse request body as type");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            };

            log::debug!(
                "[PostSensorData] Raw user_api_id bytes: {:?}, Raw sensor_api_id bytes: {:?}",
                parsed_body.user_api_id.as_bytes(),
                parsed_body.sensor_api_id.as_bytes()
            );

            let sanitized_user_api_id = parsed_body.user_api_id.replace('\0', "");
            let sanitized_sensor_api_id = parsed_body.sensor_api_id.replace('\0', "");

            match db::user_api_id_matches_sensor_api_id(
                &sanitized_user_api_id,
                &sanitized_sensor_api_id,
            ) {
                Ok(false) => {
                    log::warn!(
                        "[PostSensorData] [User API ID] does not match [Sensor API ID]: [{}] != [{}]",
                        sanitized_user_api_id,
                        sanitized_sensor_api_id
                    );
                    let mut response = Response::new(empty());
                    *response.status_mut() = PostSensorResponseCode::Unauthorized.into();
                    return Ok(response);
                }
                Ok(true) => {
                    // Just continue
                }
                Err(e) => {
                    log::error!(
                        "[PostSensorData] Calling user_api_id_matches_sensor_api_id(\nuser_api_id:{},\nsensor_api_id:{}\nERROR: {:?})",
                        sanitized_user_api_id,
                        sanitized_sensor_api_id,
                        e
                    );
                    let mut response = Response::new(empty());
                    *response.status_mut() = PostSensorResponseCode::Unauthorized.into();
                    return Ok(response);
                }
            }

            let e = match db::get_sensor_kind_from_id(&sanitized_sensor_api_id) {
                Ok(kind) => match kind {
                    SensorKind::Aht10 => {
                        let data = models::NewAht10Data {
                            sensor: sanitized_sensor_api_id,
                            serialized_data: &serialized_data,
                            added_at: parsed_body.added_at.unwrap_or_else(|| {
                                // Use current time in seconds if not provided
                                chrono::Utc::now().naive_utc()
                            }),
                        };
                        db::save_new_aht10_data(data)
                    }
                    SensorKind::Scd4x => {
                        let data = models::NewScd4xData {
                            sensor: sanitized_sensor_api_id,
                            serialized_data: &serialized_data,
                            added_at: parsed_body.added_at.unwrap_or_else(|| {
                                // Use current time in seconds if not provided
                                chrono::Utc::now().naive_utc()
                            }),
                        };
                        db::save_new_scd4x_data(data)
                    }
                },
                Err(err) => {
                    log::error!("[PostSensorData] Error getting sensor kind: {:?}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            };

            match e {
                Err(e) => {
                    log::error!("[PostSensorData] Error saving new sensor data: {:?}", e);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
                Ok(()) => {
                    log::info!("[PostSensorData] Returning 200 Status Code");
                    Ok(Response::new(empty()))
                }
            }
        }
        (&GetSensor::METHOD, GetSensor::PATH) => {
            log::info!("[GetSensor] Request matched GetSensor");

            let max_size = <GetSensor as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;

            let body =
                match extract_body_and_parse(req, max_size, Some(GetSensor::parse_request_body))
                    .await
                {
                    Ok(bytes) => bytes,
                    Err(ExtractError::PayloadTooLarge) => {
                        log::warn!("[GetSensor] Request body too large");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
                        return Ok(response);
                    }
                    Err(ExtractError::ErrorReceiving) => {
                        log::error!("[GetSensor] Error receiving request body");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsValue(err)) => {
                        log::warn!("[GetSensor] Failed to parse body as JSON: {}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsType) => {
                        log::warn!("[GetSensor] Failed to parse request body as type");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                };

            let query_result = db::query_aht10_data(body);

            let query_string = match query_result {
                Ok(data) => serde_json::to_string(&data),
                Err(err) => {
                    log::error!("[GetSensor] Error querying Sensor data: {:?}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            };

            match query_string {
                Ok(json_data) => {
                    log::info!("[GetSensor] Returning 200 Status Code");
                    let mut response = Response::new(full(json_data));
                    response
                        .headers_mut()
                        .append("content-type", HeaderValue::from_static("application/json"));
                    return Ok(response);
                }
                Err(err) => {
                    log::error!(
                        "[GetSensor] Error serializing return Sensor data: {:?}",
                        err
                    );
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            }
        }
        (&PostSensor::METHOD, PostSensor::PATH) => {
            log::info!("[PostSensor] Request matched GetSensor");

            let max_size = <PostSensor as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;
            // Extract the body from the request
            let body =
                match extract_body_and_parse(req, max_size, Some(PostSensor::parse_request_body))
                    .await
                {
                    Ok(bytes) => bytes,
                    Err(ExtractError::PayloadTooLarge) => {
                        log::warn!("[PostSensor] Request body too large");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
                        return Ok(response);
                    }
                    Err(ExtractError::ErrorReceiving) => {
                        log::error!("[PostSensor] Error receiving request body");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsValue(err)) => {
                        log::warn!("[PostSensor] Failed to parse body as JSON: {}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsType) => {
                        log::warn!("[PostSensor] Failed to parse request body as type");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                };

            let user_uuid = &body.user_uuid;
            let user_place_id_ = body.user_place_id;
            let sensor_kind = body.sensor_kind;
            let ble_mac_address = &body.sensor_mac;

            match db::new_sensor(user_uuid, sensor_kind, user_place_id_, ble_mac_address) {
                Ok(user_sensor_api_id) => {
                    log::info!(
                        "[PostSensor] Returning 200 Status Code: New sensor created with API ID: {}",
                        user_sensor_api_id
                    );
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
                    log::error!("[PostSensor] Error creating new sensor: {:?}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            }
        }
        (&GetLogin::METHOD, GetLogin::PATH) => {
            log::info!("[GetLogin] Request matched GetSensor");
            // Handle the login request
            let max_size = <GetLogin as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;

            let body =
                match extract_body_and_parse(req, max_size, Some(GetLogin::parse_request_body))
                    .await
                {
                    Ok(bytes) => bytes,
                    Err(ExtractError::PayloadTooLarge) => {
                        log::warn!("[GetLogin] Request body too large");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
                        return Ok(response);
                    }
                    Err(ExtractError::ErrorReceiving) => {
                        log::error!("[GetLogin] Error receiving request body");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsValue(err)) => {
                        log::warn!("[GetLogin] Failed to parse body as JSON: {}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsType) => {
                        log::warn!("[GetLogin] Failed to parse request body as type");
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::BAD_REQUEST;
                        return Ok(response);
                    }
                };

            match db::get_login(&body.username, &body.hashed_password) {
                Ok(token) => {
                    log::info!("[GetLogin] Returning 200 Status code");
                    let mut response = Response::new(full(token));
                    response
                        .headers_mut()
                        .append("content-type", HeaderValue::from_static("application/json"));
                    return Ok(response);
                }
                Err(err) => {
                    log::error!("[GetLogin] Error during login: {:?}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::UNAUTHORIZED;
                    return Ok(response);
                }
            }
        }
        // Return 404 Not Found for other routes.
        _ => {
            log::info!("[404 Not Found] Returning 200 Status Code");
            let mut not_found = Response::new(empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
