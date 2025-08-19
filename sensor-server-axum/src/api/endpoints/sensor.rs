use axum::{Json, extract::Query, routing::MethodRouter};
use chrono::NaiveDateTime;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, endpoints::authorized_sensor, route::Route, types::api_id::ApiId},
    auth::claims::Claims,
    db::{self, DbConnHolder, Error, user_sensors::Identifier},
    model::{HexValue, NewUserSensor},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUserSensor {
    pub device_id: ApiId,
    pub name: String,
    pub description: Option<String>,
    pub color: HexValue,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GetSensorEnum {
    FromSensorDeviceId(ApiId),
    FromPlaceName(String),
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GetSensor {
    #[serde(flatten)]
    pub param: GetSensorEnum,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PostSensor {
    pub place_name: String,
    pub device_id: ApiId,
    pub name: String,
    pub description: Option<String>,
    pub color: HexValue,
}

pub type DeleteSensor = GetSensorEnum;

pub struct Sensor {
    resources: Vec<Route>,
}

impl Sensor {
    pub const API_PATH: &str = "/sensor";
    pub fn new() -> Sensor {
        let mr = MethodRouter::new()
            .get(Self::sensor_get)
            .post(Self::sensor_post)
            .delete(Self::sensor_delete);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string(Self::API_PATH.to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn sensor_get(
        claims: Claims,
        mut conn: DbConnHolder,
        Query(payload): Query<GetSensor>,
    ) -> Result<Json<Vec<ApiUserSensor>>, StatusCode> {
        let user_id = db::users::get_user(
            &mut conn.0,
            db::users::Identifier::Username(&claims.username),
        )?
        .id;

        let payload = payload.param;

        let id = match &payload {
            GetSensorEnum::FromSensorDeviceId(device_id) => Identifier::SensorDeviceId(device_id),
            GetSensorEnum::FromPlaceName(name) => Identifier::PlaceNameAndUserId(name, user_id),
        };

        let vec = match db::user_sensors::get_user_sensor(&mut conn.0, id) {
            Ok(vec) => {
                let vec: Result<Vec<ApiUserSensor>, db::Error> = vec
                    .into_iter()
                    .map(|(place, sensor)| {
                        let color = db::colors::get_color_by_id(&mut conn.0, place.color_id)
                            .map_err(|e| {
                                log::error!("Could not get color from id: {e:?}");
                                db::Error::InternalError("Could not get color from id".into())
                            })?;
                        let aus = ApiUserSensor {
                            name: sensor.name,
                            description: sensor.description,
                            created_at: sensor.created_at,
                            updated_at: sensor.updated_at,
                            color: color,
                            device_id: ApiId::from_string(&sensor.device_id)
                                .expect("Should be valid"),
                        };
                        Ok(aus)
                    })
                    .collect();
                vec
            }
            Err(e) => {
                log::error!("Error on [get_user_sensor]: {e:?}");
                Err(e)
            }
        }?;
        log::trace!("Got {} sensors", vec.len());

        Ok(Json(vec))
    }

    async fn sensor_post(
        claims: Claims,
        mut conn: DbConnHolder,
        Json(payload): Json<PostSensor>,
    ) -> Result<Json<ApiUserSensor>, StatusCode> {
        log::trace!("sensor_post: {payload:?}");

        let user_id = db::users::get_user(
            &mut conn.0,
            db::users::Identifier::Username(&claims.username),
        )?
        .id;

        let color_id = db::colors::get_color_id(
            &mut conn.0,
            db::colors::Identifier::Hex(payload.color.clone()),
        )?;

        let place_id = db::user_places::get_user_place_id(
            &mut conn.0,
            db::user_places::Identifier::PlaceNameAndUserId(&payload.place_name, user_id),
        )?
        .into_iter()
        .next()
        .ok_or(Error::NotFound(
            format!(
                "Place Id not found for place {} of user {}",
                payload.place_name, user_id
            )
            .into(),
        ))?;

        let sensor = NewUserSensor {
            name: payload.name,
            description: payload.description,
            color_id,
            place_id,
            device_id: payload.device_id.to_string(),
        };

        log::trace!("NewUserSensor: {sensor:?}");
        //
        // match db::user_sensors::get_user_sensor(
        //     &mut conn.0,
        //     Identifier::SensorDeviceId(&payload.device_id),
        // ) {
        //     Ok(v) => {
        //         if !v.is_empty() {
        //             return Err(StatusCode::CONFLICT);
        //         }
        //     }
        //     Err(e) => match e {
        //         Error::NotFound(error) => {
        //             log::trace!(
        //                 "New sensor can be inserted, no other present with same device id: {error:?}"
        //             );
        //         }
        //         _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        //     },
        // }

        let res = db::user_sensors::insert_user_sensor(&mut conn.0, sensor)?;

        let res = ApiUserSensor {
            name: res.name,
            description: res.description,
            color: payload.color,
            created_at: res.created_at,
            updated_at: res.updated_at,
            device_id: ApiId::from_string(&res.device_id).map_err(|e| {
                log::error!("Error converting ApiId: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
        };

        log::trace!("Sensor created correctly: {res:?}");

        Ok(Json(res))
    }

    async fn sensor_delete(
        claims: Claims,
        mut conn: DbConnHolder,
        Json(payload): Json<DeleteSensor>,
    ) -> Result<Json<Vec<ApiUserSensor>>, StatusCode> {
        let user_id = db::users::get_user(
            &mut conn.0,
            db::users::Identifier::Username(&claims.username),
        )?
        .id;

        log::trace!("Deleting sensor: {payload:?}");

        let id = match &payload {
            DeleteSensor::FromSensorDeviceId(device_id) => {
                let _sensor = authorized_sensor(&mut conn.0, device_id, &claims.username)?;
                Identifier::SensorDeviceId(device_id)
            }
            DeleteSensor::FromPlaceName(name) => {
                log::trace!("Deleting all sensors from place {name}");
                Identifier::PlaceNameAndUserId(name, user_id)
            }
        };

        let vec = match db::user_sensors::delete_user_sensor(&mut conn.0, id) {
            Ok(vec) => {
                let vec: Result<Vec<ApiUserSensor>, db::Error> = vec
                    .into_iter()
                    .map(|(up, us)| {
                        let color =
                            db::colors::get_color_by_id(&mut conn.0, up.color_id).map_err(|e| {
                                log::error!("Could not get color from id: {e:?}");
                                db::Error::InternalError("Could not get color from id".into())
                            })?;
                        let aup = ApiUserSensor {
                            name: up.name,
                            description: up.description,
                            created_at: up.created_at,
                            updated_at: up.updated_at,
                            color: color,
                            device_id: ApiId::from_string(&us.device_id)
                                .expect("Should be valid ApiId"),
                        };
                        Ok(aup)
                    })
                    .collect();

                vec
            }
            Err(e) => {
                log::error!("Error on [get_user_sensor]: {e:?}");
                Err(e)
            }
        }?;

        log::trace!("Deleted {} sensors", vec.len());

        Ok(Json(vec))
    }
}

impl Endpoint for Sensor {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
    fn path(&self) -> &str {
        Self::API_PATH
    }
}

#[cfg(test)]
mod tests {

    use axum::{Json, extract::Query};

    use crate::{
        api::{
            endpoints::sensor::{DeleteSensor, GetSensor, GetSensorEnum, PostSensor, Sensor},
            types::api_id::ApiId,
        },
        auth::claims::Claims,
        db::{
            self, DbConnHolder, establish_connection,
            tests::{create_test_user, create_test_user_place, create_test_user_sensor},
        },
    };

    #[tokio::test]
    async fn test_get_by_api_id() {
        let mut conn = establish_connection(true).unwrap();
        let user = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);
        let user_sensor = create_test_user_sensor(&mut conn, &user_place);

        let body = GetSensorEnum::FromSensorDeviceId(
            ApiId::from_string(&user_sensor.device_id).expect("Valid"),
        );

        let claims = Claims {
            username: user.username,
            iat: chrono::Utc::now().timestamp() as usize,
            exp: (chrono::Utc::now()
                .checked_add_days(chrono::Days::new(3))
                .expect("Should be able to add days"))
            .timestamp() as usize,
        };

        let res_body =
            Sensor::sensor_get(claims, DbConnHolder(conn), Query(GetSensor { param: body }))
                .await
                .expect("Should not fail");

        assert!(
            res_body.len() == 1,
            "res_body.len(): {}\nres_body: {:?}",
            res_body.len(),
            res_body
        );
        assert_eq!(
            res_body.first().unwrap().device_id,
            ApiId::from_string(&user_sensor.device_id).expect("ApiId valid")
        );
    }

    #[tokio::test]
    async fn test_get_by_place_api_id() {
        let mut conn = establish_connection(true).unwrap();
        let user = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);
        let user_sensor = create_test_user_sensor(&mut conn, &user_place);

        let body = GetSensorEnum::FromPlaceName(user_place.name.clone());

        let claims = Claims {
            username: user.username,
            iat: chrono::Utc::now().timestamp() as usize,
            exp: (chrono::Utc::now()
                .checked_add_days(chrono::Days::new(3))
                .expect("Should be able to add days"))
            .timestamp() as usize,
        };

        let res_body =
            Sensor::sensor_get(claims, DbConnHolder(conn), Query(GetSensor { param: body }))
                .await
                .expect("Should not fail");

        assert!(
            res_body.len() == 1,
            "res_body.len(): {}\nres_body: {:?}",
            res_body.len(),
            res_body
        );
        assert_eq!(
            res_body.first().unwrap().device_id,
            ApiId::from_string(&user_sensor.device_id).expect("ApiId valid")
        );
    }

    #[tokio::test]
    async fn test_post() {
        let mut conn = establish_connection(true).unwrap();
        let user = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);

        let payload = PostSensor {
            place_name: place.name.clone(),
            name: "My New Awesome Sensor".to_string(),
            description: Some("A description for the new sensor.".to_string()),
            color: "#FF0000".to_string(),
            device_id: ApiId::random(),
        };

        let claims = Claims {
            username: user.username,
            iat: chrono::Utc::now().timestamp() as usize,
            exp: (chrono::Utc::now()
                .checked_add_days(chrono::Days::new(3))
                .expect("Should be able to add days"))
            .timestamp() as usize,
        };

        let res_body = Sensor::sensor_post(claims, DbConnHolder(conn), Json(payload.clone()))
            .await
            .expect("Should create a new place successfully");

        assert_eq!(res_body.name, payload.name);
        assert_eq!(res_body.description, payload.description);
        assert_eq!(res_body.color, payload.color);
        assert_eq!(res_body.device_id, payload.device_id);
    }

    #[tokio::test]
    async fn test_delete() {
        let mut conn = establish_connection(true).unwrap();
        let user = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);
        let user_sensor = create_test_user_sensor(&mut conn, &user_place);
        let sensor_to_delete_device_id =
            ApiId::from_string(&user_sensor.device_id).expect("ApiId should be valid");

        let payload = DeleteSensor::FromSensorDeviceId(sensor_to_delete_device_id.clone());

        let claims = Claims {
            username: user.username,
            iat: chrono::Utc::now().timestamp() as usize,
            exp: (chrono::Utc::now()
                .checked_add_days(chrono::Days::new(3))
                .expect("Should be able to add days"))
            .timestamp() as usize,
        };

        let deleted_sensors_response =
            Sensor::sensor_delete(claims, DbConnHolder(conn), Json(payload))
                .await
                .expect("Delete should not fail");

        assert_eq!(
            deleted_sensors_response.len(),
            1,
            "Expected to delete exactly one sensor"
        );
        assert_eq!(
            deleted_sensors_response.0.first().unwrap().device_id,
            sensor_to_delete_device_id
        );

        let mut conn_for_verify = establish_connection(true).unwrap();
        let result_after_delete = db::user_sensors::get_user_sensor(
            &mut conn_for_verify,
            db::user_sensors::Identifier::SensorDeviceId(&sensor_to_delete_device_id),
        )
        .expect("Get operation should not fail");

        assert!(
            result_after_delete.is_empty(),
            "The place should not exist in the database after being deleted."
        );
    }
}
