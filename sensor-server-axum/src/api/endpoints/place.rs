use axum::{Json, routing::MethodRouter};
use chrono::NaiveDateTime;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    auth::claims::Claims,
    db::{self, DbConnHolder, user_places::Identifier},
    model::{HexValue, NewUserPlace},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUserPlace {
    pub name: String,
    pub description: Option<String>,
    pub color: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum GetPlace {
    FromPlaceName(String),
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

impl Endpoint for Place {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
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
        let user_id = db::users::get_user(
            &mut conn.0,
            db::users::Identifier::Username(&claims.username),
        )?
        .id;

        let id = match &payload {
            GetPlace::FromPlaceName(name) => Identifier::PlaceNameAndUserId(name, user_id),
            GetPlace::UserPlaces => Identifier::UserId(user_id),
        };

        println!("in function, user.id: {id:?}");

        let vec = match db::user_places::get_user_place(&mut conn.0, id) {
            Ok(vec) => {
                let vec: Result<Vec<ApiUserPlace>, db::Error> = vec
                    .into_iter()
                    .map(|up| {
                        let color =
                            db::colors::get_color_by_id(&mut conn.0, up.color_id).map_err(|e| {
                                log::error!("Could not get color from id: {e:?}");
                                db::Error::InternalError("Could not get color from id".into())
                            })?;
                        let aup = ApiUserPlace {
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
        let user_id = db::users::get_user(
            &mut conn.0,
            db::users::Identifier::Username(&claims.username),
        )?
        .id;

        let color_id = db::colors::get_color_id(
            &mut conn.0,
            db::colors::Identifier::Hex(payload.color.clone()),
        )?;

        let place = NewUserPlace {
            user_id,
            name: payload.name,
            description: payload.description,
            color_id,
        };

        let res = db::user_places::insert_user_place(&mut conn.0, place)?;

        let res = ApiUserPlace {
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
        let user_id = db::users::get_user(
            &mut conn.0,
            db::users::Identifier::Username(&claims.username),
        )?
        .id;

        let id = match &payload {
            DeletePlace::FromPlaceName(name) => Identifier::PlaceNameAndUserId(name, user_id),
            DeletePlace::UserPlaces => Identifier::UserId(user_id),
        };

        let vec = match db::user_places::delete_user_place(&mut conn.0, id) {
            Ok(vec) => {
                let vec: Result<Vec<ApiUserPlace>, db::Error> = vec
                    .into_iter()
                    .map(|up| {
                        let color =
                            db::colors::get_color_by_id(&mut conn.0, up.color_id).map_err(|e| {
                                log::error!("Could not get color from id: {e:?}");
                                db::Error::InternalError("Could not get color from id".into())
                            })?;
                        let aup = ApiUserPlace {
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

#[cfg(test)]
mod tests {

    use axum::Json;

    use crate::{
        api::endpoints::place::{GetPlace, Place, PostPlace},
        auth::claims::Claims,
        db::{
            DbConnHolder, establish_connection,
            tests::{create_test_user, create_test_user_place},
        },
    };

    #[tokio::test]
    async fn test_place_get_user_places() {
        let body = GetPlace::UserPlaces;

        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);

        let claims = Claims {
            username: user.username,
            iat: chrono::Utc::now().timestamp() as usize,
            exp: (chrono::Utc::now()
                .checked_add_days(chrono::Days::new(3))
                .expect("Should be able to add days"))
            .timestamp() as usize,
        };
        println!("in test, user.id: {}", user.id);

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
        assert_eq!(res_body.first().unwrap().name, user_place.name);
    }

    #[tokio::test]
    async fn test_place_get_api_id() {
        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);

        let body = GetPlace::FromPlaceName(user_place.name.clone());

        let claims = Claims {
            username: user.username,
            iat: chrono::Utc::now().timestamp() as usize,
            exp: (chrono::Utc::now()
                .checked_add_days(chrono::Days::new(3))
                .expect("Should be able to add days"))
            .timestamp() as usize,
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
        assert_eq!(res_body.first().unwrap().name, user_place.name);
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
            iat: chrono::Utc::now().timestamp() as usize,
            exp: (chrono::Utc::now()
                .checked_add_days(chrono::Days::new(3))
                .expect("Should be able to add days"))
            .timestamp() as usize,
        };
        let res_body = Place::place_post(claims, DbConnHolder(conn), Json(payload.clone()))
            .await
            .expect("Should create a new place successfully");

        assert_eq!(res_body.name, payload.name);
        assert_eq!(res_body.description, payload.description);
        assert_eq!(res_body.color, payload.color);
    }

    #[tokio::test]
    async fn test_place_delete() {
        let mut conn = establish_connection().unwrap();
        let user = create_test_user(&mut conn);
        let place_to_delete = create_test_user_place(&mut conn, &user);

        let payload = GetPlace::FromPlaceName(place_to_delete.name.clone());

        let claims = Claims {
            username: user.username,
            iat: chrono::Utc::now().timestamp() as usize,
            exp: (chrono::Utc::now()
                .checked_add_days(chrono::Days::new(3))
                .expect("Should be able to add days"))
            .timestamp() as usize,
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
            deleted_places_response.0.first().unwrap().name,
            place_to_delete.name
        );
    }
}
