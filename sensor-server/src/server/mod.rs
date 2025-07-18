use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::{Method, Request, Response, StatusCode};
use sensor_lib::api::ApiEndpoint;
use sensor_lib::api::endpoints::get_login::{GetLogin, GetLoginResponseCode};
use sensor_lib::api::endpoints::get_sensor_data::{GetSensorData, GetSensorDataResponseCode};
use sensor_lib::api::endpoints::post_sensor::{PostSensor, PostSensorResponseBody};
use sensor_lib::api::endpoints::post_sensor_data::{PostSensorData, PostSensorResponseCode};
use sensor_lib::api::model::sensor_kind::SensorKind;

use crate::db;
use crate::{helper::*, models};

fn user_n_sensor_api_ids_check(
    endpoint_name: &str,
    user_api_id: &str,
    sensor_api_id: &str,
    unauthorized_code: http::StatusCode,
) -> Result<(), Response<BoxBody<Bytes, hyper::Error>>> {
    match db::user_api_id_matches_sensor_api_id(&user_api_id, &sensor_api_id) {
        Ok(false) => {
            log::warn!(
                "[{}] [User API ID] does not match [Sensor API ID]: [{}] != [{}]",
                endpoint_name,
                user_api_id,
                sensor_api_id
            );
            let mut response = Response::new(empty());
            *response.status_mut() = unauthorized_code;
            return Err(response);
        }
        Ok(true) => Ok(()),
        Err(e) => {
            log::error!(
                "[{}] Calling user_api_id_matches_sensor_api_id(\nuser_api_id:{},\nsensor_api_id:{}\nERROR: {:?})",
                endpoint_name,
                user_api_id,
                sensor_api_id,
                e
            );
            let mut response = Response::new(empty());
            *response.status_mut() = unauthorized_code;
            return Err(response);
        }
    }
}

pub async fn server(
    req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::info!(
        "\n\n\nServing method: {}, at path: {}",
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

            let (body, serialized_data) = match extract_body_and_parse(
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

            match user_n_sensor_api_ids_check(
                "[PostSensorData]",
                &body.user_api_id.as_str(),
                &body.sensor_api_id.as_str(),
                PostSensorResponseCode::Unauthorized.into(),
            ) {
                Err(r) => return Ok(r),
                Ok(_) => (),
            }

            let e = match db::get_sensor_kind_from_id(&body.user_api_id.as_str()) {
                Ok(kind) => match kind {
                    SensorKind::Aht10 => {
                        let data = models::NewAht10Data {
                            sensor: body.user_api_id.to_string(),
                            serialized_data: &serialized_data,
                            added_at: body.added_at.unwrap_or_else(|| {
                                // Use current time in seconds if not provided
                                chrono::Utc::now().naive_utc()
                            }),
                        };
                        db::save_new_aht10_data(data)
                    }
                    SensorKind::Scd4x => {
                        let data = models::NewScd4xData {
                            sensor: body.user_api_id.to_string(),
                            serialized_data: &serialized_data,
                            added_at: body.added_at.unwrap_or_else(|| {
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
        (&GetSensorData::METHOD, GetSensorData::PATH) => {
            log::info!("[GetSensorData] Request matched GetSensorData");

            let max_size = <GetSensorData as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;

            let body = match extract_body_and_parse(
                req,
                max_size,
                Some(GetSensorData::parse_request_body),
            )
            .await
            {
                Ok(bytes) => bytes,
                Err(ExtractError::PayloadTooLarge) => {
                    log::warn!("[GetSensorData] Request body too large");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
                    return Ok(response);
                }
                Err(ExtractError::ErrorReceiving) => {
                    log::error!("[GetSensorData] Error receiving request body");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsValue(err)) => {
                    log::warn!("[GetSensorData] Failed to parse body as JSON: {}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsType) => {
                    log::warn!("[GetSensorData] Failed to parse request body as type");
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(response);
                }
            };

            match user_n_sensor_api_ids_check(
                "[GetSensorData]",
                body.user_api_id.as_str(),
                body.sensor_api_id.as_str(),
                GetSensorDataResponseCode::Unauthorized.into(),
            ) {
                Err(r) => return Ok(r),
                Ok(_) => (),
            }

            let query_result = db::query_sensor_data(body);

            let query_string = match query_result {
                Ok(data) => serde_json::to_string(&data),
                Err(err) => {
                    log::error!("[GetSensorData] Error querying Sensor data: {:?}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            };

            match query_string {
                Ok(json_data) => {
                    log::info!("[GetSensorData] Returning 200 Status Code");
                    let mut response = Response::new(full(json_data));
                    response
                        .headers_mut()
                        .append("content-type", HeaderValue::from_static("application/json"));
                    return Ok(response);
                }
                Err(err) => {
                    log::error!(
                        "[GetSensorData] Error serializing return Sensor data: {:?}",
                        err
                    );
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            }
        }
        (&PostSensor::METHOD, PostSensor::PATH) => {
            log::info!("[PostSensor] Request matched PostSensor");

            let return_found_ok = |user_sensor_api_id: String| {
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
                return response;
            };

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

            let user_api_id = &body.user_api_id;
            let user_place_id = body.user_place_id;
            let sensor_kind = body.sensor_kind;
            let device_id = &body.device_id;

            match db::sensor_exists(user_api_id.as_str(), user_place_id, device_id.as_str()) {
                Ok(opt) => match opt {
                    Some(api_id) => return Ok(return_found_ok(api_id)),
                    None => {
                        log::info!("[PostSensor] sensor_exists -> None (sensor doesnt exist)")
                    }
                },
                Err(e) => {
                    log::error!(
                        "[PostSensor] on sensor_exists, continuing to new_sensor: {:?}",
                        e
                    )
                }
            }

            match db::new_sensor(
                user_api_id.as_str(),
                sensor_kind,
                user_place_id,
                device_id.as_str(),
            ) {
                Ok(user_sensor_api_id) => {
                    return Ok(return_found_ok(user_sensor_api_id));
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
            log::info!("[GetLogin] Request matched GetSensorData");
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
                        *response.status_mut() = GetLoginResponseCode::PayloadTooLarge.into();
                        return Ok(response);
                    }
                    Err(ExtractError::ErrorReceiving) => {
                        log::error!("[GetLogin] Error receiving request body");
                        let mut response = Response::new(empty());
                        *response.status_mut() = GetLoginResponseCode::InternalServerError.into();
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsValue(err)) => {
                        log::warn!("[GetLogin] Failed to parse body as JSON: {}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = GetLoginResponseCode::BadRequest.into();
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsType) => {
                        log::warn!("[GetLogin] Failed to parse request body as type");
                        let mut response = Response::new(empty());
                        *response.status_mut() = GetLoginResponseCode::BadRequest.into();
                        return Ok(response);
                    }
                };

            match db::get_login(&body.username, &body.hashed_password) {
                Ok(token) => match token {
                    Some(token) => {
                        log::info!("[GetLogin] Returning 200 Status code");
                        let mut response = Response::new(full(token));
                        response
                            .headers_mut()
                            .append("content-type", HeaderValue::from_static("application/json"));
                        Ok(response)
                    }
                    None => {
                        log::warn!("[GetLogin] get_login returned NONE (UNAUTHORIZED)");
                        let mut response = Response::new(empty());
                        *response.status_mut() = GetLoginResponseCode::Unauthorized.into();
                        Ok(response)
                    }
                },
                Err(err) => {
                    log::error!("[GetLogin] Internal Error during login: {:?}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = GetLoginResponseCode::InternalServerError.into();
                    Ok(response)
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
