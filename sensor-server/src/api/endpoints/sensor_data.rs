use axum::{extract::Query, routing::MethodRouter};
use axum_extra::extract::CookieJar;
use axum_serde_valid::Json;
use chrono::{NaiveDateTime, TimeDelta, Utc};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_valid::{Validate, json::json};
use ts_rs::TS;

use crate::{
    RoutePath,
    api::{
        Endpoint,
        endpoints::session::ApiSession,
        route::Route,
        types::{ApiTimestamp, validate::device_id::DeviceId},
    },
    auth::claims::Claims,
    db::{
        DbConnHolder,
        sensor_data::{Identifier, get_sensor_data, insert_sensor_data},
        user_sensors::AuthorizedSensor,
    },
    model::NewSensorData,
};

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor_data/")]
pub struct ApiSensorData {
    #[validate(max_length = 500)]
    pub data: String,
    pub added_at: ApiTimestamp,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor_data/")]
// WARN: Dont accept this in any endpoint
pub struct PostSensorDataResponse {
    pub api_data: ApiSensorData,
    pub new_session: ApiSession,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor_data/")]
pub struct GetSensorData {
    pub device_id: DeviceId,
    // Not included if added_at == [upper | lowest]_added_at
    pub lowest_added_at: Option<ApiTimestamp>,
    pub upper_added_at: Option<ApiTimestamp>,
}

#[derive(TS, Clone, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor_data/")]
pub struct PostSensorData {
    pub device_id: DeviceId,
    #[validate(max_length = 500)]
    pub serialized_data: String,
    pub created_at: Option<ApiTimestamp>,
}

pub struct SensorData {
    resources: Vec<Route>,
}

enum RangeDelimiter {
    Top,
    Bottom,
}

impl SensorData {
    pub const API_PATH: &str = "/sensor_data";
    pub fn new() -> SensorData {
        let mr = MethodRouter::new()
            .get(Self::sensor_data_get)
            .post(Self::sensor_data_post);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string(Self::API_PATH.to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    /// ## Max
    /// - if true, will set returned timestamp to at most the reference_utc for max
    /// - if false, will set returned timestamp to at least reference_utc for !max
    /// Also, if timestamp is None, it will return the reference_utc
    fn convert_opt_timestamp_into_naive(
        timestamp: Option<ApiTimestamp>,
        max: RangeDelimiter,
    ) -> Result<NaiveDateTime, StatusCode> {
        use chrono::DateTime;

        let reference_utc = match max {
            RangeDelimiter::Top => Utc::now()
                .checked_add_signed(TimeDelta::minutes(1))
                .ok_or_else(|| {
                    log::error!("Could not construct max_utc on convert_opt_timestamp_into_naive");
                    StatusCode::INTERNAL_SERVER_ERROR
                })?,
            RangeDelimiter::Bottom => DateTime::from_timestamp(0, 0).ok_or_else(|| {
                log::error!("Could not construct max_utc on convert_opt_timestamp_into_naive");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
        };

        let reference_cmp = match max {
            RangeDelimiter::Top => i64::min,
            RangeDelimiter::Bottom => i64::max,
        };

        let res = match timestamp {
            Some(secs) => {
                let secs = reference_cmp(secs as i64, reference_utc.timestamp());
                DateTime::from_timestamp(secs, 0)
                    .and_then(|dt| Some(dt.naive_utc()))
                    .ok_or_else(|| {
                        log::error!(
                            "Could not construct return naive on convert_opt_timestamp_into_naive from secs {secs}"
                        );
                        StatusCode::INTERNAL_SERVER_ERROR
                    })
            }
            None => Ok(reference_utc.naive_utc()),
        };

        log::trace!("convert_opt_timestamp_into_naive returned {res:?}");

        res
    }

    pub async fn sensor_data_get(
        claims: Claims,
        mut conn: DbConnHolder,
        Query(payload): Query<GetSensorData>,
    ) -> Result<Json<Vec<ApiSensorData>>, StatusCode> {
        let conn = &mut conn.0;
        let sensor = AuthorizedSensor::new(conn, &payload.device_id, &claims.username)?;

        log::trace!("Getting data for sensor: {sensor:?}");

        let low = Self::convert_opt_timestamp_into_naive(
            payload.lowest_added_at,
            RangeDelimiter::Bottom,
        )?;
        let up =
            Self::convert_opt_timestamp_into_naive(payload.upper_added_at, RangeDelimiter::Top)?;

        let range = low..up;

        log::trace!("Range was: {range:?}");

        let sensor = sensor.get();

        let sensor_data: Vec<ApiSensorData> =
            get_sensor_data(conn, Identifier::SensorId(sensor.id), low..up)?
                .into_iter()
                .map(|d| ApiSensorData {
                    data: d.data.to_string(),
                    added_at: d.added_at.and_utc().timestamp() as ApiTimestamp,
                })
                .collect();

        log::trace!("Returning {} datums", sensor_data.len());

        Ok(Json(sensor_data))
    }

    pub async fn sensor_data_post(
        jar: CookieJar,
        claims: Claims,
        mut conn: DbConnHolder,
        Json(payload): Json<PostSensorData>,
    ) -> Result<(CookieJar, Json<PostSensorDataResponse>), StatusCode> {
        let conn = &mut conn.0;

        let sensor = AuthorizedSensor::new(conn, &payload.device_id, &claims.username)?;

        log::trace!(
            "Adding data to sensor {sensor:?}, data: {:?}",
            payload.serialized_data
        );

        let added_at = payload
            .created_at
            .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp as i64, 0))
            .and_then(|date| Some(date.naive_utc()));

        let sensor = sensor.get();

        let new_data = NewSensorData {
            sensor_id: sensor.id,
            data: json!(payload.serialized_data),
            added_at,
        };

        let data = insert_sensor_data(conn, new_data)?;

        let api_data = ApiSensorData {
            data: data.data.to_string(),
            added_at: data.added_at.and_utc().timestamp() as usize,
        };

        let claims = Claims::new(claims.username);

        let new_session = ApiSession::from_claims(claims).map_err(|e| {
            log::error!("Error generating new session from_claims: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok((
            jar.add(new_session.build_cookie()),
            Json(PostSensorDataResponse {
                api_data,
                new_session,
            }),
        ))
    }
}

impl Endpoint for SensorData {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
    fn path(&self) -> &str {
        Self::API_PATH
    }
}

#[cfg(test)]
mod test {
    use axum::extract::Query;
    use axum_extra::extract::CookieJar;
    use axum_serde_valid::Json;

    use crate::{
        api::{
            endpoints::sensor_data::{GetSensorData, PostSensorData, SensorData},
            types::validate::device_id::DeviceId,
        },
        auth::claims::Claims,
        db::{
            DbConnHolder, establish_connection,
            tests::{create_test_user, create_test_user_place, create_test_user_sensor},
        },
    };

    #[tokio::test]
    async fn test_get_sensor_data() {
        let mut conn_uref = establish_connection(true).unwrap();
        let conn = &mut conn_uref;

        let (user, _) = create_test_user(conn);
        let user_place = create_test_user_place(conn, &user);
        let sensor = create_test_user_sensor(conn, &user_place);

        let claims = Claims::new(user.username);

        let device_id = DeviceId::from_string(&sensor.device_id).unwrap();

        let json = GetSensorData {
            device_id,
            lowest_added_at: None,
            upper_added_at: None,
        };

        let conn = DbConnHolder(conn_uref);

        let res = SensorData::sensor_data_get(claims, conn, Query(json))
            .await
            .expect("Should not fail");

        assert!(res.len() == 0);
    }

    #[tokio::test]
    async fn test_post_sensor_data() {
        let mut conn_uref = establish_connection(true).unwrap();
        let conn = &mut conn_uref;

        let (user, _) = create_test_user(conn);
        let user_place = create_test_user_place(conn, &user);
        let sensor = create_test_user_sensor(conn, &user_place);

        let claims = Claims::new(user.username);

        let device_id = DeviceId::from_string(&sensor.device_id).unwrap();

        let json = PostSensorData {
            device_id,
            serialized_data: String::default(),
            created_at: None,
        };

        let conn = DbConnHolder(conn_uref);

        let _res = SensorData::sensor_data_post(CookieJar::new(), claims, conn, Json(json.clone()))
            .await
            .expect("Should not fail");
    }
}
