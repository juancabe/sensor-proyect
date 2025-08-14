use axum::{Json, routing::MethodRouter};
use chrono::NaiveDateTime;
use hyper::StatusCode;
use sensor_lib::api::model::api_id::ApiId;
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    db::{self, user_places::Identifier},
    middleware::extractor::{DbConnHolder, jwt::Claims},
    model::{HexValue, NewUserPlace},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUserPlace {
    pub api_id: ApiId,
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GetPlace {
    FromPlaceApiId(ApiId),
    UserPlaces,
}

type DeletePlace = GetPlace;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct PostPlace {
    name: String,
    description: Option<String>,
    color: HexValue,
}

pub struct Place {
    resources: Vec<Route>,
}

impl Place {
    pub fn new() -> Place {
        let mr = MethodRouter::new()
            .get(Self::place_get)
            .post(Self::place_post)
            .delete(Self::place_delete);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string("/places".to_string()).expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn place_get(
        claims: Claims,
        mut conn: DbConnHolder,
        Json(payload): Json<GetPlace>,
    ) -> Result<Json<Vec<ApiUserPlace>>, StatusCode> {
        let id = match payload {
            GetPlace::FromPlaceApiId(api_id) => Identifier::PlaceApiId(api_id),
            GetPlace::UserPlaces => {
                let api_id = ApiId::from_string(&claims.user_api_id).map_err(|e| {
                    log::error!("Could not construct ApiId from claims: {:?}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR;
                })?;
                Identifier::UserApiId(api_id)
            }
        };

        let vec = match db::user_places::get_user_place(&mut conn.0, id) {
            Ok(vec) => {
                if !vec.iter().all(|up| ApiId::from_string(&up.api_id).is_ok()) {
                    log::error!(
                        "[get_user_place] some of these have invalid ApiId: {:?}",
                        vec
                    );
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                let vec: Result<Vec<ApiUserPlace>, db::Error> = vec
                    .into_iter()
                    .map(|up| {
                        let color =
                            db::colors::get_color_by_id(&mut conn.0, up.color_id).map_err(|e| {
                                log::error!("Could not get color from id: {e:?}");
                                db::Error::InternalError("Could not get color from id".into())
                            })?;
                        let aup = ApiUserPlace {
                            api_id: ApiId::from_string(&up.api_id).expect("Should be valid ApiId"),
                            name: up.name,
                            description: up.description,
                            created_at: up.created_at,
                            updated_at: up.updated_at,
                            color: color,
                        };
                        Ok(aup)
                    })
                    .collect();
                vec
            }
            Err(e) => {
                log::error!("Error on [get_user_place]: {e:?}");
                Err(e)
            }
        }?;

        Ok(Json(vec))
    }

    async fn place_post(
        claims: Claims,
        mut conn: DbConnHolder,
        Json(payload): Json<PostPlace>,
    ) -> Result<Json<ApiUserPlace>, StatusCode> {
        let api_id = ApiId::random().to_string();

        let user_id = db::users::get_user_id(
            &mut conn.0,
            db::users::Identifier::ApiId(claims.user_api_id),
        )?;

        let color_id = db::colors::get_color_id(
            &mut conn.0,
            db::colors::Identifier::Hex(payload.color.clone()),
        )?;

        let place = NewUserPlace {
            api_id,
            user_id,
            name: payload.name,
            description: payload.description,
            color_id,
        };

        let res = db::user_places::insert_user_place(&mut conn.0, place)?;

        let api_id = ApiId::from_string(&res.api_id).map_err(|e| {
            log::error!("Error converting ApiId: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        let res = ApiUserPlace {
            api_id,
            name: res.name,
            description: res.description,
            color: payload.color,
            created_at: res.created_at,
            updated_at: res.updated_at,
        };

        Ok(Json(res))
    }

    async fn place_delete(
        claims: Claims,
        mut conn: DbConnHolder,
        Json(payload): Json<DeletePlace>,
    ) -> Result<Json<Vec<ApiUserPlace>>, StatusCode> {
        let id = match payload {
            DeletePlace::FromPlaceApiId(api_id) => Identifier::PlaceApiId(api_id),
            DeletePlace::UserPlaces => {
                let api_id = ApiId::from_string(&claims.user_api_id).map_err(|e| {
                    log::error!("Could not construct ApiId from claims: {:?}", e);
                    return StatusCode::INTERNAL_SERVER_ERROR;
                })?;
                Identifier::UserApiId(api_id)
            }
        };

        let vec = match db::user_places::delete_user_place(&mut conn.0, id) {
            Ok(vec) => {
                if !vec.iter().all(|up| ApiId::from_string(&up.api_id).is_ok()) {
                    log::error!(
                        "[get_user_place] some of these have invalid ApiId: {:?}",
                        vec
                    );
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
                let vec: Result<Vec<ApiUserPlace>, db::Error> = vec
                    .into_iter()
                    .map(|up| {
                        let color =
                            db::colors::get_color_by_id(&mut conn.0, up.color_id).map_err(|e| {
                                log::error!("Could not get color from id: {e:?}");
                                db::Error::InternalError("Could not get color from id".into())
                            })?;
                        let aup = ApiUserPlace {
                            api_id: ApiId::from_string(&up.api_id).expect("Should be valid ApiId"),
                            name: up.name,
                            description: up.description,
                            created_at: up.created_at,
                            updated_at: up.updated_at,
                            color: color,
                        };
                        Ok(aup)
                    })
                    .collect();
                vec
            }
            Err(e) => {
                log::error!("Error on [get_user_place]: {e:?}");
                Err(e)
            }
        }?;

        Ok(Json(vec))
    }
}

impl Endpoint for Place {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
}

#[cfg(test)]
mod tests {

    use axum::Json;
    use sensor_lib::api::model::api_id::ApiId;

    use crate::{
        api::endpoints::place::{GetPlace, Place, PostPlace},
        db::{
            self, establish_connection,
            tests::{create_test_user, create_test_user_place},
        },
        middleware::extractor::{DbConnHolder, jwt::Claims},
    };

    #[tokio::test]
    async fn test_place_get_user_places() {
        let body = GetPlace::UserPlaces;

        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);

        let claims = Claims {
            username: user.username,
            user_api_id: user.api_id,
            iat: chrono::Utc::now().timestamp(),
        };

        let res_body = Place::place_get(
            claims,
            DbConnHolder(conn),
            Json::from_bytes(
                serde_json::to_string(&body)
                    .expect("Should be serializable")
                    .as_bytes(),
            )
            .expect("Json from Json"),
        )
        .await
        .expect("Should not fail");

        assert!(
            res_body.len() == 1,
            "res_body.len(): {}\nres_body: {:?}",
            res_body.len(),
            res_body
        );
        assert_eq!(
            res_body.first().unwrap().api_id,
            ApiId::from_string(&user_place.api_id).expect("ApiId valid")
        );
    }

    #[tokio::test]
    async fn test_place_get_api_id() {
        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);

        let body = GetPlace::FromPlaceApiId(
            ApiId::from_string(&user_place.api_id).expect("Should be valid"),
        );

        let claims = Claims {
            username: user.username,
            user_api_id: user.api_id,
            iat: chrono::Utc::now().timestamp(),
        };

        let res_body = Place::place_get(
            claims,
            DbConnHolder(conn),
            Json::from_bytes(
                serde_json::to_string(&body)
                    .expect("Should be serializable")
                    .as_bytes(),
            )
            .expect("Json from Json"),
        )
        .await
        .expect("Should not fail");

        assert!(
            res_body.len() == 1,
            "res_body.len(): {}\nres_body: {:?}",
            res_body.len(),
            res_body
        );
        assert_eq!(
            res_body.first().unwrap().api_id,
            ApiId::from_string(&user_place.api_id).expect("ApiId valid")
        );
    }

    #[tokio::test]
    async fn test_place_post() {
        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);

        let payload = PostPlace {
            name: "My New Awesome Place".to_string(),
            description: Some("A description for the new place.".to_string()),
            color: "#FF0000".to_string(),
        };

        let claims = Claims {
            username: user.username,
            user_api_id: user.api_id,
            iat: chrono::Utc::now().timestamp(),
        };

        let res_body = Place::place_post(claims, DbConnHolder(conn), Json(payload.clone()))
            .await
            .expect("Should create a new place successfully");

        assert_eq!(res_body.name, payload.name);
        assert_eq!(res_body.description, payload.description);
        assert_eq!(res_body.color, payload.color);
        assert!(!res_body.api_id.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_place_delete() {
        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);
        let place_to_delete = create_test_user_place(&mut conn, &user);
        let place_to_delete_api_id =
            ApiId::from_string(&place_to_delete.api_id).expect("ApiId should be valid");

        let payload = GetPlace::FromPlaceApiId(place_to_delete_api_id.clone());

        let claims = Claims {
            username: user.username,
            user_api_id: user.api_id,
            iat: chrono::Utc::now().timestamp(),
        };

        let deleted_places_response =
            Place::place_delete(claims, DbConnHolder(conn), Json(payload))
                .await
                .expect("Delete should not fail");

        assert_eq!(
            deleted_places_response.len(),
            1,
            "Expected to delete exactly one place"
        );
        assert_eq!(
            deleted_places_response.0.first().unwrap().api_id,
            place_to_delete_api_id
        );

        let mut conn_for_verify = establish_connection().unwrap();
        let result_after_delete = db::user_places::get_user_place(
            &mut conn_for_verify,
            db::user_places::Identifier::PlaceApiId(place_to_delete_api_id),
        )
        .expect("Get operation should not fail");

        assert!(
            result_after_delete.is_empty(),
            "The place should not exist in the database after being deleted."
        );
    }
}
