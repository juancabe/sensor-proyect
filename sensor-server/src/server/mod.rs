use std::collections::HashMap;

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
use sensor_lib::api::model::sensor_kind::SensorKind;
use sensor_lib::api::{ApiEndpoint, model};

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

fn post_sensor_data(
    body: PostSensorDataRequestBody,
    serialized_data: &str,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::debug!("[PostSensorData] Received body: {:?}", body);

    match user_n_sensor_api_ids_check(
        "[PostSensorData]",
        &body.user_api_id.as_str(),
        &body.sensor_api_id.as_str(),
        PostSensorResponseCode::Unauthorized.into(),
    ) {
        Err(r) => return Ok(r),
        Ok(_) => (),
    }

    let e = match db::get_sensor_kind_from_id(&body.sensor_api_id.as_str()) {
        Ok(kind) => match kind {
            SensorKind::Aht10 => {
                let added_at = body
                    .added_at
                    .and_then(|secs| (secs as i64).checked_mul(1_000))
                    .and_then(|millis| {
                        chrono::Utc::timestamp_millis_opt(&chrono::Utc, millis).earliest()
                    })
                    .and_then(|dt| Some(dt.naive_utc()));

                let data = models::NewAht10Data {
                    sensor: body.sensor_api_id.to_string(),
                    serialized_data: &serialized_data,
                    added_at: added_at.unwrap_or_else(|| {
                        // Use current time in seconds if not provided
                        chrono::Utc::now().naive_utc()
                    }),
                };
                db::save_new_aht10_data(data)
            }
            SensorKind::Scd4x => {
                let added_at = body
                    .added_at
                    .and_then(|secs| (secs as i64).checked_mul(1_000))
                    .and_then(|millis| {
                        chrono::Utc::timestamp_millis_opt(&chrono::Utc, millis).earliest()
                    })
                    .and_then(|dt| Some(dt.naive_utc()));

                let data = models::NewScd4xData {
                    sensor: body.sensor_api_id.to_string(),
                    serialized_data: &serialized_data,
                    added_at: added_at.unwrap_or_else(|| {
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

fn get_sensor_data(
    body: GetSensorDataRequestBody,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    log::debug!("[GetSensorData] Received body: {:?}", body);

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

    match query_result {
        Ok(data) => {
            let mut vec = Vec::with_capacity(data.0.len());
            let mut failed_serialize = 0;
            for datum in data.0 {
                match serde_json::to_string(&datum) {
                    Ok(s) => vec.push(s),
                    Err(_) => failed_serialize += 1,
                }
            }

            let resp = GetSensorDataResponseBody {
                item_count: vec.len(),
                serialized_data: vec,
                failed_serialize: failed_serialize,
                failed_deserialize: data.1,
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

fn login(body: LoginRequestBody) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
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
    let mut places: HashMap<u32, (String, Option<String>)> = HashMap::new();

    let sensors = match db::get_user_sensors(&query.username, query.user_api_id.as_str()) {
        Ok(r) => r.into_iter().map(|(sens, place)| {
            places.insert(place.id as u32, (place.name, place.description));

            let (kind, api_id, device_id) = {
                let k = match SensorKind::from_i32(sens.kind) {
                    Some(k) => k,
                    None => {
                        log::error!("[PostUserSummary] Error converting DB kind to SensorKind");
                        return None;
                    }
                };
                let a = match ApiId::from_string(&sens.api_id) {
                    Ok(a) => a,
                    Err(e) => {
                        log::error!(
                            "[PostUserSummary] Error converting DB api_id to ApiId: {:?}",
                            e
                        );
                        return None;
                    }
                };
                let d = match ApiId::from_string(&sens.device_id) {
                    Ok(a) => a,
                    Err(e) => {
                        log::error!(
                            "[PostUserSummary] Error converting DB device_id to ApiId: {:?}",
                            e
                        );
                        return None;
                    }
                };
                (k, a, d)
            };

            let sum = model::user_summary::SensorSummary {
                kind,
                api_id,
                device_id,
                last_update: sens.last_measurement.and_utc().timestamp() as u32,
                place: place.id as u32,
            };

            Some(sum)
        }),
        Err(e) => match e {
            db::Error::NotFound => {
                log::warn!("[PostUserSummary] get_user_sensors returned NotFound (UNAUTHORIZED)");
                let mut response = Response::new(empty());
                *response.status_mut() = LoginResponseCode::Unauthorized.into();
                return Ok(response);
            }
            _ => {
                log::error!("[PostUserSummary] Error getting user sensors: {:?}", e);
                let mut response = Response::new(empty());
                *response.status_mut() = PostUserSummaryResponseCode::InternalServerError.into();
                return Ok(response);
            }
        },
    }
    .filter_map(|r| r)
    .collect();

    let email = match db::get_user_email(&query.username, query.user_api_id.as_str()) {
        Ok(email) => email,
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
    };

    let iter = places.into_iter().map(|(k, (name, desc))| (k, name, desc));
    let places = Vec::from_iter(iter);

    let summary = model::user_summary::UserSummary {
        username: query.username,
        email,
        sensors,
        places,
    };

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

    use super::*;
    use sensor_lib::api::model::{
        any_sensor_data::AnySensorData, api_id::ApiId, scd4x_data::Scd4xData,
    };

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
    fn test_post_sensor_data() {
        let body = PostSensorDataRequestBody {
            user_api_id: ApiId::from_string(VALID_USER_API_ID).unwrap(),
            sensor_api_id: ApiId::from_string(VALID_SCD41_API_ID).unwrap(),
            data: AnySensorData::Scd4x(Scd4xData::new("todelete".to_string(), 123, 12.2, 12.3)),
            added_at: None,
        };
        let serialized_data = serde_json::to_string(&body.data).unwrap();
        post_sensor_data(body, &serialized_data).unwrap();
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
}
