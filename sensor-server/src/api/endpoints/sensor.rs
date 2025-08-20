use axum::{extract::Query, routing::MethodRouter};
use axum_serde_valid::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

use crate::{
    RoutePath,
    api::{
        Endpoint,
        endpoints::authorized_sensor,
        route::Route,
        types::{
            ApiTimestamp,
            device_id::DeviceId,
            validate::{
                api_color::ApiColor, api_description::ApiDescription,
                api_entity_name::ApiEntityName,
            },
        },
    },
    auth::claims::Claims,
    db::{self, DbConnHolder, Error, user_sensors::Identifier},
    model::NewUserSensor,
};

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct ApiUserSensor {
    pub device_id: DeviceId,
    pub name: ApiEntityName,
    pub description: Option<ApiDescription>,
    pub color: ApiColor,
    pub created_at: ApiTimestamp,
    pub updated_at: ApiTimestamp,
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
pub enum GetSensorEnum {
    FromSensorDeviceId(DeviceId),
    FromPlaceName(ApiEntityName),
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct GetSensor {
    #[serde(flatten)]
    pub param: GetSensorEnum,
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Clone, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct PostSensor {
    pub place_name: ApiEntityName,
    pub device_id: DeviceId,
    pub name: ApiEntityName,
    pub description: Option<ApiDescription>,
    pub color: ApiColor,
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub enum DeleteSensor {
    FromSensorDeviceId(DeviceId),
    FromPlaceName(ApiEntityName),
}

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
            GetSensorEnum::FromPlaceName(name) => {
                Identifier::PlaceNameAndUserId(name.as_str(), user_id)
            }
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
                            name: sensor.name.into(),
                            description: sensor.description.map(|d| d.into()),
                            created_at: sensor.created_at.and_utc().timestamp() as usize,
                            updated_at: sensor.updated_at.and_utc().timestamp() as usize,
                            color: color.into(),
                            device_id: DeviceId::from_string(&sensor.device_id)
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
            db::colors::Identifier::Hex(payload.color.clone().into()),
        )?;

        let place_id = db::user_places::get_user_place_id(
            &mut conn.0,
            db::user_places::Identifier::PlaceNameAndUserId(&payload.place_name.as_str(), user_id),
        )?
        .into_iter()
        .next()
        .ok_or(Error::NotFound(
            format!(
                "Place Id not found for place {:?} of user {}",
                payload.place_name, user_id
            )
            .into(),
        ))?;

        let sensor = NewUserSensor {
            name: payload.name.into(),
            description: payload.description.map(|d| d.into()),
            color_id,
            place_id,
            device_id: payload.device_id.to_string(),
        };

        log::trace!("NewUserSensor: {sensor:?}");

        let res = db::user_sensors::insert_user_sensor(&mut conn.0, sensor)?;

        let res = ApiUserSensor {
            name: res.name.into(),
            description: res.description.map(|d| d.into()),
            color: payload.color.into(),
            created_at: res.created_at.and_utc().timestamp() as usize,
            updated_at: res.updated_at.and_utc().timestamp() as usize,
            device_id: DeviceId::from_string(&res.device_id).map_err(|e| {
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
                log::trace!("Deleting all sensors from place {name:?}");
                Identifier::PlaceNameAndUserId(name.as_str(), user_id)
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
                            name: up.name.into(),
                            description: up.description.map(|d| d.into()),
                            created_at: up.created_at.and_utc().timestamp() as usize,
                            updated_at: up.updated_at.and_utc().timestamp() as usize,
                            color: color.into(),
                            device_id: DeviceId::from_string(&us.device_id)
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

    use axum::extract::Query;
    use axum_serde_valid::Json;
    use serde_valid::json::ToJsonString;

    use crate::{
        api::{
            endpoints::sensor::{DeleteSensor, GetSensor, GetSensorEnum, PostSensor, Sensor},
            types::device_id::DeviceId,
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
        let (user, _) = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);
        let user_sensor = create_test_user_sensor(&mut conn, &user_place);

        let body = GetSensorEnum::FromSensorDeviceId(
            DeviceId::from_string(&user_sensor.device_id).expect("Valid"),
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
            res_body.to_json_string()
        );
        assert_eq!(
            res_body.first().unwrap().device_id,
            DeviceId::from_string(&user_sensor.device_id).expect("ApiId valid")
        );
    }

    #[tokio::test]
    async fn test_get_by_place_api_id() {
        let mut conn = establish_connection(true).unwrap();
        let (user, _) = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);
        let user_sensor = create_test_user_sensor(&mut conn, &user_place);

        let body = GetSensorEnum::FromPlaceName(user_place.name.clone().into());

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
            res_body.to_json_string()
        );
        assert_eq!(
            res_body.first().unwrap().device_id,
            DeviceId::from_string(&user_sensor.device_id).expect("ApiId valid")
        );
    }

    #[tokio::test]
    async fn test_post() {
        let mut conn = establish_connection(true).unwrap();
        let (user, _) = create_test_user(&mut conn);
        let place = create_test_user_place(&mut conn, &user);

        let payload = PostSensor {
            place_name: place.name.clone().into(),
            name: "My New Awesome Sensor".to_string().into(),
            description: Some("A description for the new sensor.".to_string().into()),
            color: "#FF0000".to_string().into(),
            device_id: DeviceId::random(),
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

        assert_eq!(res_body.name, payload.name.into());
        assert_eq!(res_body.description, payload.description.map(|d| d.into()));
        assert_eq!(res_body.color, payload.color.into());
        assert_eq!(res_body.device_id, payload.device_id);
    }

    #[tokio::test]
    async fn test_delete() {
        let mut conn = establish_connection(true).unwrap();
        let (user, _) = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);
        let user_sensor = create_test_user_sensor(&mut conn, &user_place);
        let sensor_to_delete_device_id =
            DeviceId::from_string(&user_sensor.device_id).expect("ApiId should be valid");

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
