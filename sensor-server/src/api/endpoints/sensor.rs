use axum::{extract::Query, routing::MethodRouter};
use axum_serde_valid::Json;
use common::{
    endpoints_io::{
        sensor::{
            ApiUserSensor, DeleteSensor, GetSensor, GetSensorEnum, GetSensorResponse, PostSensor,
            PutSensor, SensorChange,
        },
        sensor_data::ApiSensorData,
    },
    types::{
        ApiTimestamp,
        validate::{api_color::ApiColor, device_id::DeviceId},
    },
};
use hyper::StatusCode;

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    auth::claims::Claims,
    db::model::NewUserSensor,
    db::{
        self, DbConnHolder, Error,
        user_places::get_user_place,
        user_sensors::{AuthorizedSensor, Identifier, Update, update_user_sensor},
        users,
    },
};

// impl ApiUserSensor {
//     pub fn from_sensor_place_color(
//         sensor: UserSensor,
//         color: String,
//         place_name: String,
//     ) -> Result<Self, device_id::Error> {
//         Ok(Self {
//             device_id: DeviceId::from_string(&sensor.device_id)?,
//             name: sensor.name.into(),
//             description: sensor.description.map(|d| d.into()),
//             color: ApiColor::from(color),
//             created_at: sensor.created_at.and_utc().timestamp() as ApiTimestamp,
//             updated_at: sensor.updated_at.and_utc().timestamp() as ApiTimestamp,
//             place_name: place_name.into(),
//         })
//     }
// }

pub struct Sensor {
    resources: Vec<Route>,
}

impl Sensor {
    pub const API_PATH: &str = "/sensor";
    pub fn new() -> Sensor {
        let mr = MethodRouter::new()
            .get(Self::sensor_get)
            .post(Self::sensor_post)
            .put(Self::sensor_put)
            .delete(Self::sensor_delete);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string(Self::API_PATH.to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn sensor_put(
        claims: Claims,
        mut conn: DbConnHolder,
        Json(PutSensor { device_id, change }): Json<PutSensor>,
    ) -> Result<Json<ApiUserSensor>, StatusCode> {
        let conn = &mut conn.0;

        let auth_sensor = AuthorizedSensor::from_username(conn, &device_id, &claims.username)?;
        let user_id = users::get_user(conn, users::Identifier::Username(&claims.username))?.id;
        let sensor = update_user_sensor(conn, auth_sensor, change.clone() as Update, user_id)?;

        let place_name = if let SensorChange::PlaceName(name) = change {
            name
        } else {
            get_user_place(conn, db::user_places::Identifier::UserId(user_id))?
                .into_iter()
                .filter(|place| place.id == sensor.place_id)
                .next()
                .ok_or_else(|| {
                    log::error!("The place wasn't found after updating a sensor");
                    Error::NotFound("Place not found".into())
                })?
                .name
                .into()
        };

        let color_id = sensor.color_id;
        Ok(Json(ApiUserSensor {
            device_id: DeviceId::from_string(&sensor.device_id).map_err(|e| {
                log::error!("Could not construct DeviceId: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?,
            name: sensor.name.into(),
            description: sensor.description.map(|d| d.into()),
            color: ApiColor::from(db::colors::get_color_by_id(conn, color_id).map_err(|e| {
                log::error!("Could not get color!: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?),
            created_at: sensor.created_at.and_utc().timestamp() as ApiTimestamp,
            updated_at: sensor.updated_at.and_utc().timestamp() as ApiTimestamp,
            place_name: place_name.into(),
            pub_key: sensor.pub_key.into(),
        }))
    }

    async fn sensor_get(
        claims: Claims,
        mut conn: DbConnHolder,
        Query(payload): Query<GetSensor>,
    ) -> Result<Json<Vec<GetSensorResponse>>, StatusCode> {
        let user_id = db::users::get_user(
            &mut conn.0,
            db::users::Identifier::Username(&claims.username),
        )?
        .id;

        let payload = payload.param;

        let id = match &payload {
            GetSensorEnum::FromSensorDeviceId(device_id) => Identifier::SensorDeviceId(
                AuthorizedSensor::from_username(&mut conn.0, device_id, &claims.username)?,
            ),
            GetSensorEnum::FromPlaceName(name) => {
                Identifier::PlaceNameAndUserId(name.as_str(), user_id)
            }
        };

        let vec = match db::user_sensors::get_user_sensor_and_place_and_last_data(&mut conn.0, id) {
            Ok(vec) => {
                let vec: Result<Vec<GetSensorResponse>, db::Error> = vec
                    .into_iter()
                    .map(|(place, sensor, data)| {
                        let color = db::colors::get_color_by_id(&mut conn.0, sensor.color_id)
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
                            place_name: place.name.into(),
                            pub_key: sensor.pub_key.into(),
                        };

                        let data = data.map(|d| ApiSensorData {
                            data: d.data.to_string(),
                            added_at: d.added_at.and_utc().timestamp() as ApiTimestamp,
                        });

                        Ok(GetSensorResponse {
                            sensor: aus,
                            last_data: data,
                        })
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

        let sensor = NewUserSensor::new(
            place_id,
            payload.device_id.to_string(),
            payload.pub_key.clone().into(),
            payload.name.into(),
            payload.description.map(|d| d.into()),
            color_id,
        )
        .map_err(|e| {
            log::warn!("User provided a wrong pub_key and it passed api filters: {e:?}");
            StatusCode::BAD_REQUEST
        })?;

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
            place_name: payload.place_name,
            pub_key: payload.pub_key,
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
            DeleteSensor::FromSensorDeviceId(device_id) => Identifier::SensorDeviceId(
                AuthorizedSensor::from_username(&mut conn.0, device_id, &claims.username)?,
            ),
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
                        let aus = ApiUserSensor {
                            name: us.name.into(),
                            description: us.description.map(|d| d.into()),
                            created_at: us.created_at.and_utc().timestamp() as usize,
                            updated_at: us.updated_at.and_utc().timestamp() as usize,
                            color: color.into(),
                            device_id: DeviceId::from_string(&us.device_id)
                                .expect("Should be valid ApiId"),
                            place_name: up.name.into(),
                            pub_key: us.pub_key.into(),
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
    use common::types::validate::{api_pub_key::ApiPubKey, device_id::DeviceId};

    use crate::{
        api::endpoints::sensor::{DeleteSensor, GetSensor, GetSensorEnum, PostSensor, Sensor},
        auth::claims::{Claims, get_new_id},
        db::{
            DbConnHolder, establish_connection,
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
            jwt_id: get_new_id(),
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

        assert!(res_body.len() == 1, "res_body.len(): {}", res_body.len(),);
        assert_eq!(
            res_body.first().unwrap().sensor.device_id,
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
            jwt_id: get_new_id(),
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

        assert!(res_body.len() == 1, "res_body.len(): {}\n", res_body.len(),);
        assert_eq!(
            res_body.first().unwrap().sensor.device_id,
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
            pub_key: ApiPubKey::random(&[123u8; 32]),
        };

        let claims = Claims {
            jwt_id: get_new_id(),
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
            jwt_id: get_new_id(),
            username: user.username.clone(),
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
    }
}
