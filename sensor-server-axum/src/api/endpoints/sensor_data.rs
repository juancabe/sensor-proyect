use axum::{Json, routing::MethodRouter};
use chrono::NaiveDateTime;
use hyper::StatusCode;
use jsonwebtoken::{Header, encode};
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{
        Endpoint,
        route::Route,
        types::{ApiTimestamp, api_id::ApiId},
    },
    auth::{claims::Claims, keys::KEYS},
    db::{
        self, DbConn, DbConnHolder, Error,
        sensor_data::{Identifier, get_sensor_data, insert_sensor_data},
    },
    model::{NewSensorData, UserSensor},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSensorData {
    pub data: serde_json::value::Value,
    pub added_at: ApiTimestamp,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostSensorDataResponse {
    pub api_data: ApiSensorData,
    pub new_jwt: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GetSensorData {
    device_id: ApiId,
    lowest_added_at: Option<ApiTimestamp>,
    upper_added_at: Option<ApiTimestamp>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PostSensorData {
    pub device_id: ApiId,
    pub serialized_data: serde_json::Value,
    pub created_at: Option<ApiTimestamp>,
}

pub struct SensorData {
    resources: Vec<Route>,
}

impl SensorData {
    pub fn new() -> SensorData {
        let mr = MethodRouter::new()
            .get(Self::sensor_data_get)
            .post(Self::sensor_data_post);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string("/sensor_data".to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    fn authorized_sensor(
        conn: &mut DbConn,
        device_id: &ApiId,
        username: &str,
    ) -> Result<UserSensor, StatusCode> {
        let db_res = db::user_sensors::get_user_sensor(
            conn,
            db::user_sensors::Identifier::SensorDeviceId(device_id),
        )?;

        let (place, sensor) = db_res
            .into_iter()
            .next()
            .ok_or(Error::NotFound("NotFound".into()))?;

        let user_id = db::users::get_user(conn, db::users::Identifier::Username(username))?.id;

        if user_id != place.user_id {
            Err(StatusCode::UNAUTHORIZED)
        } else {
            Ok(sensor)
        }
    }

    fn convert_opt_timestamp_into_naive(
        timestamp: Option<ApiTimestamp>,
    ) -> Result<NaiveDateTime, StatusCode> {
        use chrono::DateTime;
        match timestamp {
            Some(secs) => {
                let secs = i64::min(secs as i64, chrono::Utc::now().timestamp());
                DateTime::from_timestamp(secs, 0)
                    .and_then(|dt| Some(dt.naive_utc()))
                    .ok_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            None => Ok(NaiveDateTime::default()),
        }
    }

    pub async fn sensor_data_get(
        claims: Claims,
        mut conn: DbConnHolder,
        Json(payload): Json<GetSensorData>,
    ) -> Result<Json<Vec<ApiSensorData>>, StatusCode> {
        let conn = &mut conn.0;
        let sensor = Self::authorized_sensor(conn, &payload.device_id, &claims.username)?;

        let up = Self::convert_opt_timestamp_into_naive(payload.upper_added_at)?;
        let low = Self::convert_opt_timestamp_into_naive(payload.lowest_added_at)?;

        let sensor_data = get_sensor_data(conn, Identifier::SensorId(sensor.id), low..up)?
            .into_iter()
            .map(|d| ApiSensorData {
                data: d.data,
                added_at: d.added_at.and_utc().timestamp() as ApiTimestamp,
            })
            .collect();

        Ok(Json(sensor_data))
    }

    pub async fn sensor_data_post(
        claims: Claims,
        mut conn: DbConnHolder,
        Json(payload): Json<PostSensorData>,
    ) -> Result<Json<PostSensorDataResponse>, StatusCode> {
        let conn = &mut conn.0;
        let db_res = db::user_sensors::get_user_sensor(
            conn,
            db::user_sensors::Identifier::SensorDeviceId(&payload.device_id),
        )?;

        let (place, sensor) = db_res.first().ok_or(Error::NotFound("NotFound".into()))?;

        let user_id =
            db::users::get_user(conn, db::users::Identifier::Username(&claims.username))?.id;

        if user_id != place.user_id {
            Err(StatusCode::UNAUTHORIZED)?
        }

        let added_at = payload
            .created_at
            .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp as i64, 0))
            .and_then(|date| Some(date.naive_utc()));

        let new_data = NewSensorData {
            sensor_id: sensor.id,
            data: payload.serialized_data,
            added_at,
        };

        let data = insert_sensor_data(conn, new_data)?;

        let api_data = ApiSensorData {
            data: data.data,
            added_at: data.added_at.and_utc().timestamp() as usize,
        };

        let claims = Claims::new(claims.username);

        let new_jwt = encode(&Header::default(), &claims, &KEYS.encoding).map_err(|e| {
            log::error!("Error encoding JWT from {claims:?}: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(Json(PostSensorDataResponse { api_data, new_jwt }))
    }
}

impl Endpoint for SensorData {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
}

#[cfg(test)]
mod test {
    use axum::Json;

    use crate::{
        api::{
            endpoints::sensor_data::{GetSensorData, PostSensorData, SensorData},
            types::api_id::ApiId,
        },
        auth::claims::Claims,
        db::{
            DbConnHolder, establish_connection,
            tests::{create_test_user, create_test_user_place, create_test_user_sensor},
        },
    };

    #[tokio::test]
    async fn test_get_sensor_data() {
        let mut conn_uref = establish_connection().unwrap();
        let conn = &mut conn_uref;

        let user = create_test_user(conn);
        let user_place = create_test_user_place(conn, &user);
        let sensor = create_test_user_sensor(conn, &user_place);

        let claims = Claims::new(user.username);

        let device_id = ApiId::from_string(&sensor.device_id).unwrap();

        let json = GetSensorData {
            device_id,
            lowest_added_at: None,
            upper_added_at: None,
        };

        let conn = DbConnHolder(conn_uref);

        let res = SensorData::sensor_data_get(claims, conn, Json(json))
            .await
            .expect("Should not fail");

        assert!(res.len() == 0);
    }

    #[tokio::test]
    async fn test_post_sensor_data() {
        let mut conn_uref = establish_connection().unwrap();
        let conn = &mut conn_uref;

        let user = create_test_user(conn);
        let user_place = create_test_user_place(conn, &user);
        let sensor = create_test_user_sensor(conn, &user_place);

        let claims = Claims::new(user.username);

        let device_id = ApiId::from_string(&sensor.device_id).unwrap();

        let json = PostSensorData {
            device_id,
            serialized_data: serde_json::Value::default(),
            created_at: None,
        };

        let conn = DbConnHolder(conn_uref);

        let res = SensorData::sensor_data_post(claims, conn, Json(json.clone()))
            .await
            .expect("Should not fail");

        assert_eq!(res.api_data.data, json.serialized_data);
    }
}
