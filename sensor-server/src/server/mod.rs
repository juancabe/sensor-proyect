use chrono::TimeZone;
use http_body_util::combinators::BoxBody;
use hyper::body::Bytes;
use hyper::header::HeaderValue;
use hyper::{Method, Request, Response, StatusCode};
use sensor_lib::api::endpoints::get_sensor_data::{
    GetSensorData, GetSensorDataRequestBody, GetSensorDataResponseBody, GetSensorDataResponseCode,
};
use sensor_lib::api::endpoints::login::{
    Login, LoginRequestBody, LoginResponseBody, LoginResponseCode,
};
use sensor_lib::api::endpoints::post_place::{
    PostPlace, PostPlaceRequestBody, PostPlaceResponseBody, PostPlaceResponseCode,
};
use sensor_lib::api::endpoints::post_sensor::{
    PostSensor, PostSensorRequestBody, PostSensorResponseBody,
};
use sensor_lib::api::endpoints::post_sensor_data::{
    PostSensorData, PostSensorDataRequestBody, PostSensorResponseCode,
};
use sensor_lib::api::endpoints::post_user_summary::{
    PostUserSummary, PostUserSummaryRequestBody, PostUserSummaryResponseBody,
    PostUserSummaryResponseCode,
};
use sensor_lib::api::endpoints::register::{
    Register, RegisterRequestBody, RegisterResponseBody, RegisterResponseCode,
};
use sensor_lib::api::model::api_id::ApiId;
use sensor_lib::api::model::color_palette::{PlaceColor, SensorColor};
use sensor_lib::api::model::sensor_kind::SensorKind;
use sensor_lib::api::model::user_summary::PlaceSummary;
use sensor_lib::api::{ApiEndpoint, model};

use crate::db::{self, delete_place, delete_sensor};
use crate::{helper::*, models};

fn user_n_sensor_api_ids_check(
    endpoint_name: &str,
    user_api_id: &ApiId,
    sensor_api_id: &ApiId,
    unauthorized_code: http::StatusCode,
) -> Result<(), Response<BoxBody<Bytes, hyper::Error>>> {
    match db::user_api_id_matches_sensor_api_id(user_api_id, sensor_api_id) {
        Ok(false) => {
            log::warn!(
                "[{}] [User API ID] does not match [Sensor API ID]: [{:?}] != [{:?}]",
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
                "[{}] Calling user_api_id_matches_sensor_api_id(\nuser_api_id:{:?},\nsensor_api_id:{:?}\nERROR: {:?})",
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

fn post_sensor_data(
    body: PostSensorDataRequestBody,
    serialized_data: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::debug!("[PostSensorData] Received body: {:?}", body);

    match user_n_sensor_api_ids_check(
        "[PostSensorData]",
        &body.user_api_id,
        &body.sensor_api_id,
        PostSensorResponseCode::Unauthorized.into(),
    ) {
        Err(r) => return Ok(r),
        Ok(_) => (),
    }

    let added_at = body
        .added_at
        .and_then(|secs| (secs as i64).checked_mul(1_000))
        .and_then(|millis| chrono::Utc::timestamp_millis_opt(&chrono::Utc, millis).earliest())
        .and_then(|dt| Some(dt.naive_utc()));
    let added_at = match added_at {
        Some(t) => t,
        None => chrono::Utc::now().naive_utc(),
    };

    let data = models::NewSensorData {
        sensor: body.sensor_api_id.to_string(),
        serialized_data: &serialized_data,
        added_at: added_at,
    };

    match db::save_new_sensor_data(data) {
        Err(e) => {
            log::error!("[PostSensorData] Error saving new sensor data: {:?}", e);
            let mut response = Response::new(empty());
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(response);
        }
        Ok(()) => (),
    }

    match db::update_sensor_last_measurement(added_at, &body.sensor_api_id.as_str()) {
        Ok(()) => {
            log::info!("[PostSensorData] Returning 200 Status Code");
            Ok(Response::new(empty()))
        }
        Err(e) => {
            log::error!("[PostSensorData] Error updating last_measurement: {:?}", e);
            let mut response = Response::new(empty());
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(response);
        }
    }
}

fn get_sensor_data(
    body: GetSensorDataRequestBody,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::debug!("[GetSensorData] Received body: {:?}", body);

    match user_n_sensor_api_ids_check(
        "[GetSensorData]",
        &body.user_api_id,
        &body.sensor_api_id,
        GetSensorDataResponseCode::Unauthorized.into(),
    ) {
        Err(r) => return Ok(r),
        Ok(_) => (),
    }

    let query_result = db::query_sensor_data(body);

    match query_result {
        Ok((kind, data)) => {
            let data: Vec<(String, u32)> = data
                .into_iter()
                .map(|d| (d.serialized_data, d.added_at.and_utc().timestamp() as u32))
                .collect();

            let resp = GetSensorDataResponseBody {
                item_count: data.len(),
                serialized_data: data,
                sensor_kind: kind,
            };

            let s = serde_json::to_string(&resp).expect("Should be serializable");

            log::info!("[GetSensorData] Returning 200 Status Code");
            let mut response = Response::new(full(s));
            response
                .headers_mut()
                .append("content-type", HeaderValue::from_static("application/json"));
            return Ok(response);
        }
        Err(err) => {
            log::error!("[GetSensorData] Error querying Sensor data: {:?}", err);
            if let db::Error::NotFound = err {
                log::error!("INCONSISTENCY, SHOULD HAVE ALREADY CHECK WETHER UNAUTHORIZED")
            };
            let mut response = Response::new(empty());
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(response);
        }
    }
}

fn post_sensor(
    body: PostSensorRequestBody,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::debug!("[PostSensor] Received body: {:?}", body);

    let return_found_ok = |user_sensor_api_id: String| {
        log::info!(
            "[PostSensor] Returning 200 Status Code: New sensor created with API ID: {}",
            user_sensor_api_id
        );
        let response_body = PostSensorResponseBody {
            sensor_api_id: user_sensor_api_id.clone(),
        };
        let response_body_json =
            serde_json::to_string(&response_body).expect("Failed to serialize response body");

        let mut response = Response::new(full(response_body_json));
        response
            .headers_mut()
            .append("content-type", HeaderValue::from_static("application/json"));
        return response;
    };

    match body {
        PostSensorRequestBody::CreateSensor {
            user_api_id,
            user_place_id,
            device_id,
            sensor_kind,
            sensor_name,
            sensor_description,
            sensor_color,
        } => {
            match db::sensor_exists(&user_api_id, &user_place_id, &device_id) {
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
                &user_api_id,
                sensor_kind,
                &user_place_id,
                &device_id,
                &sensor_name,
                sensor_description.as_deref(),
                sensor_color,
            ) {
                Ok(user_sensor_api_id) => {
                    return Ok(return_found_ok(user_sensor_api_id));
                }
                Err(err) => match err {
                    db::Error::NotFound => {
                        log::warn!(
                            "[PostSensor] Error creating new sensor (UNAUTHORIZED): {:?}",
                            err
                        );
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::UNAUTHORIZED;
                        return Ok(response);
                    }
                    _ => {
                        log::error!("[PostSensor] Error creating new sensor: {:?}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        return Ok(response);
                    }
                },
            }
        }
        PostSensorRequestBody::DeleteSensor {
            user_api_id,
            sensor_api_id,
        } => match delete_sensor(&user_api_id, &sensor_api_id) {
            Ok(()) => {
                log::info!("[PostSensor] Returning 200 Status Code: Sensor deleted");
                let mut response = Response::new(empty());
                *response.status_mut() = StatusCode::OK;
                return Ok(response);
            }
            Err(e) => match e {
                db::Error::NotFound => {
                    log::warn!("[PostSensor] Error deleting sensor (UNAUTHORIZED): {:?}", e);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::UNAUTHORIZED;
                    return Ok(response);
                }
                _ => {
                    log::error!("[PostSensor] Error deleting sensor: {:?}", e);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    return Ok(response);
                }
            },
        },
    }
}

fn post_place(
    body: PostPlaceRequestBody,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::debug!("[PostPlace] Received body: {:?}", body);

    match body {
        PostPlaceRequestBody::Create {
            username,
            user_api_id,
            place_name,
            place_description,
            place_color,
        } => {
            match db::new_place(
                &user_api_id,
                &username,
                &place_name,
                place_description.as_deref(),
                place_color,
            ) {
                Ok(place_id) => {
                    log::info!(
                        "[PostPlace] Returning 200 Status Code: New place created with ID: {:?}",
                        place_id
                    );
                    let response_body = PostPlaceResponseBody::Created {
                        place_id: place_id,
                        place_name: place_name,
                        place_description: place_description,
                    };
                    let response_body_json = serde_json::to_string(&response_body)
                        .expect("Failed to serialize response body");

                    let mut response = Response::new(full(response_body_json));
                    response
                        .headers_mut()
                        .append("content-type", HeaderValue::from_static("application/json"));
                    Ok(response)
                }
                Err(err) => match err {
                    db::Error::NotFound => {
                        log::warn!(
                            "[PostPlace] Error creating new place (UNAUTHORIZED): {:?}",
                            err
                        );
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::UNAUTHORIZED;
                        Ok(response)
                    }
                    _ => {
                        log::error!("[PostPlace] Error creating new place: {:?}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        Ok(response)
                    }
                },
            }
        }
        PostPlaceRequestBody::Delete {
            user_api_id,
            place_id,
        } => match delete_place(&user_api_id, &place_id) {
            Ok(()) => {
                log::info!("[PostPlace] Successfully deleted place");
                let mut response = Response::new(empty());
                *response.status_mut() = StatusCode::NO_CONTENT;
                Ok(response)
            }
            Err(e) => match e {
                db::Error::NotFound => {
                    log::warn!("[PostPlace] Error deleting place (UNAUTHORIZED): {:?}", e);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::UNAUTHORIZED;
                    Ok(response)
                }
                _ => {
                    log::error!("[PostPlace] Error deleting place: {:?}", e);
                    let mut response = Response::new(empty());
                    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                    Ok(response)
                }
            },
        },
    }
}

fn login(body: LoginRequestBody) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::debug!(
        "[Login] called with username ({}) and hashed password ({})",
        &body.username,
        &body.hashed_password
    );
    match db::get_login(&body.username, &body.hashed_password) {
        Ok(api_id) => match api_id {
            Some(api_id) => {
                log::info!("[Login] Returning 200 Status code");
                let resp_body = LoginResponseBody { api_id };
                let resp_body = serde_json::to_string(&resp_body).expect("Should be serializable");
                let mut response = Response::new(full(resp_body));
                response
                    .headers_mut()
                    .append("content-type", HeaderValue::from_static("application/json"));
                Ok(response)
            }
            None => {
                log::warn!("[Login] login returned NONE (UNAUTHORIZED)");
                let mut response = Response::new(empty());
                *response.status_mut() = LoginResponseCode::Unauthorized.into();
                Ok(response)
            }
        },
        Err(err) => {
            log::error!("[Login] Internal Error during login: {:?}", err);
            let mut response = Response::new(empty());
            *response.status_mut() = LoginResponseCode::InternalServerError.into();
            Ok(response)
        }
    }
}

fn register(
    query: RegisterRequestBody,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    match db::new_user(query) {
        Ok(api_id) => {
            log::info!("[Register] Returning 200 Status code");
            let resp = RegisterResponseBody::Correct(api_id);
            let mut response = Response::new(full(
                serde_json::to_string(&resp).expect("Should be serializable"),
            ));
            response
                .headers_mut()
                .append("content-type", HeaderValue::from_static("application/json"));
            Ok(response)
        }
        Err(err) => match err {
            db::NewUserError::EmailUsed => {
                log::warn!("[Register] Returning 200 Status code (EmailUsed)");
                let resp = RegisterResponseBody::Incorrect(
                    sensor_lib::api::endpoints::register::RegisterIncorrectReason::EmailUsed,
                );
                let mut response = Response::new(full(
                    serde_json::to_string(&resp).expect("Should be serializable"),
                ));
                response
                    .headers_mut()
                    .append("content-type", HeaderValue::from_static("application/json"));
                Ok(response)
            }

            db::NewUserError::UsernameUsed => {
                log::warn!("[Register] Returning 200 Status code (UsernameUsed)");
                let resp = RegisterResponseBody::Incorrect(
                    sensor_lib::api::endpoints::register::RegisterIncorrectReason::UsernameUsed,
                );
                let mut response = Response::new(full(
                    serde_json::to_string(&resp).expect("Should be serializable"),
                ));
                response
                    .headers_mut()
                    .append("content-type", HeaderValue::from_static("application/json"));
                Ok(response)
            }

            db::NewUserError::OtherError(error) => {
                log::error!("[Register] Internal Error during login: {:?}", error);
                let mut response = Response::new(empty());
                *response.status_mut() = RegisterResponseCode::InternalServerError.into();
                Ok(response)
            }
        },
    }
}

fn post_user_summary(
    query: PostUserSummaryRequestBody,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::debug!(
        "post_user_summary called with username: {}",
        &query.username
    );

    // check if user exists and get email
    let email = match db::get_user_email(&query.username, query.user_api_id.as_str()) {
        Ok(email) => email,
        Err(e) => match e {
            db::Error::NotFound => {
                // If Unauthorized, should have returned earlier
                log::warn!(
                    "[PostUserSummary] Error getting user email (UNAUTHORIZED): {:?}",
                    e
                );
                let mut response = Response::new(empty());
                *response.status_mut() = PostUserSummaryResponseCode::Unauthorized.into();
                return Ok(response);
            }
            _ => {
                // If Unauthorized, should have returned earlier
                log::error!("[PostUserSummary] Error getting user email: {:?}", e);
                let mut response = Response::new(empty());
                *response.status_mut() = PostUserSummaryResponseCode::InternalServerError.into();
                return Ok(response);
            }
        },
    };

    let mut places: Vec<PlaceSummary> = Vec::new();

    match db::get_user_places(&query.username, &query.user_api_id.as_str()) {
        Ok(vector) => {
            for place in vector {
                let place_id = match ApiId::from_string(&place.api_id) {
                    Ok(api_id) => api_id,
                    Err(e) => {
                        log::error!(
                            "[PostUserSummary] Inconsistent PLACE API_ID found in DB, place.api_id: {}, error: {:?}",
                            place.api_id,
                            e
                        );
                        continue;
                    }
                };

                let color = match PlaceColor::from_str(&place.color) {
                    Some(c) => c,
                    None => {
                        log::error!(
                            "[PostUserSummary] Inconsistent PLACE COLOR found in DB, place.color: {}",
                            place.color
                        );
                        continue;
                    }
                };

                places.push(PlaceSummary {
                    place_id,
                    last_update: place.updated_at.and_utc().timestamp() as u32,
                    name: place.name,
                    description: place.description,
                    color,
                });
            }
        }
        Err(e) => {
            log::error!("[PostUserSummary] Error getting user places: {:?}", e);
            let mut response = Response::new(empty());
            *response.status_mut() = PostUserSummaryResponseCode::InternalServerError.into();
            return Ok(response);
        }
    }

    let sensors = match db::get_user_sensors(&query.username, query.user_api_id.as_str()) {
        Ok(r) => r.into_iter().map(|(sens, place)| {
            let (place_api_id, kind, sensor_api_id, device_id, sensor_color) = match (
                ApiId::from_string(&place.api_id),
                SensorKind::from_i32(sens.kind),
                ApiId::from_string(&sens.api_id),
                ApiId::from_string(&sens.device_id),
                SensorColor::from_str(&sens.color),
            ) {
                (
                    Ok(place_api_id),
                    Some(kind),
                    Ok(sensor_api_id),
                    Ok(device_id),
                    Some(sensor_color),
                ) => (place_api_id, kind, sensor_api_id, device_id, sensor_color),
                _ => {
                    log::error!(
                        "[PostUserSummary] INCONSISTENCY Error converting DB values to API types:
                    ApiId::from_string({}), -> {:?}
                    SensorKind::from_i32({}), -> {:?}
                    ApiId::from_string({}), -> {:?}
                    ApiId::from_string({}), -> {:?}
                    SensorColor::from_str({}), -> {:?}
                    ",
                        &place.api_id,
                        ApiId::from_string(&place.api_id),
                        sens.kind,
                        SensorKind::from_i32(sens.kind),
                        &sens.api_id,
                        ApiId::from_string(&sens.api_id),
                        &sens.device_id,
                        ApiId::from_string(&sens.device_id),
                        &sens.color,
                        SensorColor::from_str(&sens.color),
                    );
                    return None; // Skip this sensor if any conversion fails
                }
            };

            let sum = model::user_summary::SensorSummary {
                kind,
                api_id: sensor_api_id,
                device_id,
                last_update: sens.last_measurement.and_utc().timestamp() as u32,
                place_id: place_api_id,
                name: sens.name,
                description: sens.description,
                color: sensor_color,
            };

            Some(sum)
        }),
        Err(e) => {
            // If Unauthorized, should have returned earlier
            log::error!("[PostUserSummary] Error getting user email: {:?}", e);
            if let db::Error::NotFound = e {
                log::warn!("[PostUserSummary]        ***** ASSERTION FAILED *****       ");
                log::warn!("[PostUserSummary] Unauthorized, should have returned earlier");
                log::warn!("[PostUserSummary]        ***** ASSERTION FAILED *****       ");
            }
            let mut response = Response::new(empty());
            *response.status_mut() = PostUserSummaryResponseCode::InternalServerError.into();
            return Ok(response);
        }
    }
    .filter_map(|r| r)
    .collect();

    let summary = model::user_summary::UserSummary {
        username: query.username,
        email,
        sensors,
        places,
    };

    log::debug!("Summary created: {:?}", summary);

    log::info!("[PostUserSummary] Returning 200 Status code");
    let resp = PostUserSummaryResponseBody { summary };
    let mut response = Response::new(full(
        serde_json::to_string(&resp).expect("Should be serializable"),
    ));
    response
        .headers_mut()
        .append("content-type", HeaderValue::from_static("application/json"));
    Ok(response)
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
            log::debug!("[PostSensorData] Request: {:?}", req);

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

            post_sensor_data(body, &serialized_data)
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

            get_sensor_data(body)
        }
        (&PostSensor::METHOD, PostSensor::PATH) => {
            log::info!("[PostSensor] Request matched PostSensor");

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

            post_sensor(body)
        }
        (&Login::METHOD, Login::PATH) => {
            log::info!("[Login] Request matched Login");
            // Handle the login request
            let max_size = <Login as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;

            let body = match extract_body_and_parse(req, max_size, Some(Login::parse_request_body))
                .await
            {
                Ok(bytes) => bytes,
                Err(ExtractError::PayloadTooLarge) => {
                    log::warn!("[Login] Request body too large");
                    let mut response = Response::new(empty());
                    *response.status_mut() = LoginResponseCode::PayloadTooLarge.into();
                    return Ok(response);
                }
                Err(ExtractError::ErrorReceiving) => {
                    log::error!("[Login] Error receiving request body");
                    let mut response = Response::new(empty());
                    *response.status_mut() = LoginResponseCode::InternalServerError.into();
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsValue(err)) => {
                    log::warn!("[Login] Failed to parse body as JSON: {}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = LoginResponseCode::BadRequest.into();
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsType) => {
                    log::warn!("[Login] Failed to parse request body as type");
                    let mut response = Response::new(empty());
                    *response.status_mut() = LoginResponseCode::BadRequest.into();
                    return Ok(response);
                }
            };

            login(body)
        }
        (&Register::METHOD, Register::PATH) => {
            log::info!("[Register] Request matched Register");
            // Handle the login request
            let max_size = <Register as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;
            let body =
                match extract_body_and_parse(req, max_size, Some(Register::parse_request_body))
                    .await
                {
                    Ok(bytes) => bytes,
                    Err(ExtractError::PayloadTooLarge) => {
                        log::warn!("[Login] Request body too large");
                        let mut response = Response::new(empty());
                        *response.status_mut() = LoginResponseCode::PayloadTooLarge.into();
                        return Ok(response);
                    }
                    Err(ExtractError::ErrorReceiving) => {
                        log::error!("[Login] Error receiving request body");
                        let mut response = Response::new(empty());
                        *response.status_mut() = LoginResponseCode::InternalServerError.into();
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsValue(err)) => {
                        log::warn!("[Login] Failed to parse body as JSON: {}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = LoginResponseCode::BadRequest.into();
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsType) => {
                        log::warn!("[Login] Failed to parse request body as type");
                        let mut response = Response::new(empty());
                        *response.status_mut() = LoginResponseCode::BadRequest.into();
                        return Ok(response);
                    }
                };

            register(body)
        }
        (&PostUserSummary::METHOD, PostUserSummary::PATH) => {
            log::info!("[PostUserSummary] Request matched PostUserSummary");
            // Handle the login request
            let max_size = <PostUserSummary as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;

            let body = match extract_body_and_parse(
                req,
                max_size,
                Some(PostUserSummary::parse_request_body),
            )
            .await
            {
                Ok(bytes) => bytes,
                Err(ExtractError::PayloadTooLarge) => {
                    log::warn!("[PostUserSummary] Request body too large");
                    let mut response = Response::new(empty());
                    *response.status_mut() = PostUserSummaryResponseCode::PayloadTooLarge.into();
                    return Ok(response);
                }
                Err(ExtractError::ErrorReceiving) => {
                    log::error!("[PostUserSummary] Error receiving request body");
                    let mut response = Response::new(empty());
                    *response.status_mut() =
                        PostUserSummaryResponseCode::InternalServerError.into();
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsValue(err)) => {
                    log::warn!("[PostUserSummary] Failed to parse body as JSON: {}", err);
                    let mut response = Response::new(empty());
                    *response.status_mut() = PostUserSummaryResponseCode::BadRequest.into();
                    return Ok(response);
                }
                Err(ExtractError::ParseErrorAsType) => {
                    log::warn!("[PostUserSummary] Failed to parse request body as type");
                    let mut response = Response::new(empty());
                    *response.status_mut() = PostUserSummaryResponseCode::BadRequest.into();
                    return Ok(response);
                }
            };

            post_user_summary(body)
        }
        (&PostPlace::METHOD, PostPlace::PATH) => {
            log::info!("[PostPlace] Request matched PostPlace");
            // Handle the login request
            let max_size = <PostPlace as ApiEndpoint<'_, '_>>::MAX_REQUEST_BODY_SIZE;

            let body =
                match extract_body_and_parse(req, max_size, Some(PostPlace::parse_request_body))
                    .await
                {
                    Ok(bytes) => bytes,
                    Err(ExtractError::PayloadTooLarge) => {
                        log::warn!("[PostPlace] Request body too large");
                        let mut response = Response::new(empty());
                        *response.status_mut() = PostPlaceResponseCode::PayloadTooLarge.into();
                        return Ok(response);
                    }
                    Err(ExtractError::ErrorReceiving) => {
                        log::error!("[PostPlace] Error receiving request body");
                        let mut response = Response::new(empty());
                        *response.status_mut() = PostPlaceResponseCode::InternalServerError.into();
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsValue(err)) => {
                        log::warn!("[PostPlace] Failed to parse body as JSON: {}", err);
                        let mut response = Response::new(empty());
                        *response.status_mut() = PostPlaceResponseCode::BadRequest.into();
                        return Ok(response);
                    }
                    Err(ExtractError::ParseErrorAsType) => {
                        log::warn!("[PostPlace] Failed to parse request body as type");
                        let mut response = Response::new(empty());
                        *response.status_mut() = PostPlaceResponseCode::BadRequest.into();
                        return Ok(response);
                    }
                };

            post_place(body)
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

#[cfg(test)]
mod test {

    // These tests will need to diesel migration redo everytime they are run

    const VALID_USER_API_ID: &'static str = "94a990533d76aaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const VALID_AHT10_API_ID: &'static str = "94a990533d761111111111111111111111111111";
    const VALID_SCD41_API_ID: &'static str = "94a990533d762222222222222222222222222222";
    const VALID_PLACE_ID_1: &'static str = "94a990533d76ffaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const VALID_PLACE_ID_2: &'static str = "94a990533d76fffaaaaaaaaaaaaaaaaaaaaaaaaa";

    use super::*;
    use sensor_lib::api::model::{
        any_sensor_data::AnySensorData, api_id::ApiId, scd4x_data::Scd4xData,
    };

    #[test]
    fn init() {
        env_logger::init();
        log::info!("Logger initialized for tests");
    }

    #[test]
    fn test_post_place() {
        let body = PostPlaceRequestBody::Create {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            username: "testuser".to_string(),
            place_name: "Test Place".to_string(),
            place_description: Some("A place for testing".to_string()),
            place_color: PlaceColor::HEX_402E2A,
        };
        let response = post_place(body).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_post_place_unexistent_user() {
        let body = PostPlaceRequestBody::Create {
            user_api_id: ApiId::random(),
            username: "unexistentuser".to_string(),
            place_name: "Test Place".to_string(),
            place_description: Some("A place for testing".to_string()),
            place_color: PlaceColor::HEX_957E78,
        };
        let response = post_place(body).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_post_user_summary() {
        let body = PostUserSummaryRequestBody {
            username: "testuser".to_string(),
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
        };
        let response = post_user_summary(body).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_post_user_summary_unexistent_user() {
        let body = PostUserSummaryRequestBody {
            username: "unexistentuser".to_string(),
            user_api_id: ApiId::random(),
        };
        let response = post_user_summary(body).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_post_sensor() {
        let body = PostSensorRequestBody::CreateSensor {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            user_place_id: ApiId::from_string(VALID_PLACE_ID_1).unwrap(),
            sensor_kind: SensorKind::Scd4x,
            device_id: ApiId::random(),
            sensor_name: "Sensor Server 3".to_string(),
            sensor_description: Some("A sensor for testing".to_string()),
            sensor_color: SensorColor::HEX_6FF0D1,
        };
        let response = post_sensor(body).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_post_sensor_unexistent_user() {
        let body = PostSensorRequestBody::CreateSensor {
            user_api_id: ApiId::random(),
            user_place_id: ApiId::from_string(VALID_PLACE_ID_2).unwrap(),
            sensor_kind: SensorKind::Scd4x,
            device_id: ApiId::random(),
            sensor_name: "Sensor Server 2".to_string(),
            sensor_description: Some("A sensor for testing".to_string()),
            sensor_color: SensorColor::HEX_6FF0D1,
        };
        let response = post_sensor(body).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_post_sensor_unexistent_place() {
        let body = PostSensorRequestBody::CreateSensor {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            user_place_id: ApiId::random(), // Non-existent place ID
            sensor_kind: SensorKind::Scd4x,
            device_id: ApiId::random(),
            sensor_name: "Sensor Server 1".to_string(),
            sensor_description: None,
            sensor_color: SensorColor::HEX_6FF0D1,
        };
        let response = post_sensor(body).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_register() {
        let body = RegisterRequestBody {
            username: "newuser".to_string(),
            email: "newuser@example.com".to_string(),
            hashed_password: "hashed_password".to_string(),
        };
        let response = register(body).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_register_existing_username() {
        let body = RegisterRequestBody {
            username: "testuser".to_string(),
            email: "existinguser@example.com".to_string(),
            hashed_password: "hashed_password".to_string(),
        };
        let response = register(body).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_register_then_login() {
        let register_body = RegisterRequestBody {
            username: "loginuser".to_string(),
            email: "loginuser@example.com".to_string(),
            hashed_password: "hashed_password".to_string(),
        };
        let response = register(register_body).unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let login_body = LoginRequestBody {
            username: "loginuser".to_string(),
            hashed_password: "hashed_password".to_string(),
        };
        let response = login(login_body).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_login() {
        let body = LoginRequestBody {
            username: "testuser".to_string(),
            hashed_password: "ae5deb822e0d71992900471a7199d0d95b8e7c9d05c40a8245a281fd2c1d6684"
                .to_string(),
        };
        let response = login(body).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_login_invalid_credentials() {
        let body = LoginRequestBody {
            username: "testuser".to_string(),
            hashed_password: "invalid_password".to_string(),
        };
        let response = login(body).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_post_sensor_data() {
        let body = PostSensorDataRequestBody {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            sensor_api_id: ApiId::from_string(VALID_SCD41_API_ID).unwrap(),
            data: AnySensorData::Scd4x(Scd4xData::new("todelete".to_string(), 123, 12.2, 12.3)),
            added_at: None,
        };
        let serialized_data = serde_json::to_string(&body.data).unwrap();
        let response = post_sensor_data(body, &serialized_data).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_post_sensor_data_unexistent_user() {
        let body = PostSensorDataRequestBody {
            user_api_id: ApiId::random(),
            sensor_api_id: ApiId::from_string(VALID_SCD41_API_ID).unwrap(),
            data: AnySensorData::Scd4x(Scd4xData::new("todelete".to_string(), 123, 12.2, 12.3)),
            added_at: None,
        };
        let serialized_data = serde_json::to_string(&body.data).unwrap();
        let response = post_sensor_data(body, &serialized_data).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // AI
    #[test]
    fn test_post_sensor_data_unexistent_sensor() {
        let body = PostSensorDataRequestBody {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            sensor_api_id: ApiId::random(),
            data: AnySensorData::Scd4x(Scd4xData::new("todelete".to_string(), 123, 12.2, 12.3)),
            added_at: None,
        };
        let serialized_data = serde_json::to_string(&body.data).unwrap();
        let response = post_sensor_data(body, &serialized_data).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_get_sensor_data() {
        let body = GetSensorDataRequestBody {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            sensor_api_id: ApiId::from_string(VALID_SCD41_API_ID).unwrap(),
            added_at_upper: None,
            added_at_lower: None,
        };
        let response = get_sensor_data(body).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_get_sensor_data_unexistent_user() {
        let body = GetSensorDataRequestBody {
            user_api_id: ApiId::random(),
            sensor_api_id: ApiId::from_string(VALID_SCD41_API_ID).unwrap(),
            added_at_upper: None,
            added_at_lower: None,
        };
        let response = get_sensor_data(body).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_register_existing_email() {
        let body = RegisterRequestBody {
            username: "anotheruser".to_string(),
            email: "testuser@example.com".to_string(), // Existing email
            hashed_password: "hashed_password".to_string(),
        };
        let response = register(body).unwrap();
        // The server responds with OK, but the body indicates the error.
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_delete_sensor_unauthorized() {
        let body = PostSensorRequestBody::DeleteSensor {
            user_api_id: ApiId::random(), // Unauthorized user
            sensor_api_id: ApiId::from_string(VALID_AHT10_API_ID).unwrap(),
        };
        let response = post_sensor(body).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_delete_sensor_not_found() {
        let body = PostSensorRequestBody::DeleteSensor {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            sensor_api_id: ApiId::random(), // Non-existent sensor
        };
        let response = post_sensor(body).unwrap();
        // The authorization check fails for a non-existent sensor, returning UNAUTHORIZED.
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_delete_place_unauthorized() {
        let body = PostPlaceRequestBody::Delete {
            user_api_id: ApiId::random(), // Unauthorized user
            place_id: ApiId::from_string(VALID_PLACE_ID_1).unwrap(),
        };
        let response = post_place(body).unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn test_delete_place_not_found() {
        let body = PostPlaceRequestBody::Delete {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            place_id: ApiId::random(), // Non-existent place
        };
        let response = post_place(body).unwrap();
        // The handler maps a database NotFound error to UNAUTHORIZED.
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
