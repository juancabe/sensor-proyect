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
        route::Route,
        types::{
            ApiTimestamp,
            validate::{
                api_color::ApiColor, api_description::ApiDescription,
                api_entity_name::ApiEntityName,
            },
        },
    },
    auth::claims::Claims,
    db::{
        self, DbConnHolder,
        user_places::{Identifier, Update, update_user_place},
    },
    model::{NewUserPlace, UserPlace},
};

#[derive(Debug, Serialize, Deserialize, TS, Validate)]
#[ts(export, export_to = "./api/endpoints/place/")]
pub struct ApiUserPlace {
    #[validate]
    pub name: ApiEntityName,
    #[validate]
    pub description: Option<ApiDescription>,
    #[validate]
    pub color: ApiColor,
    pub created_at: ApiTimestamp,
    pub updated_at: ApiTimestamp,
}

impl ApiUserPlace {
    pub fn from_user_place_and_color(place: UserPlace, color: String) -> Self {
        Self {
            name: place.name.into(),
            description: place.description.map(|d| d.into()),
            color: color.into(),
            created_at: place.created_at.and_utc().timestamp() as ApiTimestamp,
            updated_at: place.updated_at.and_utc().timestamp() as ApiTimestamp,
        }
    }
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Clone, Validate)]
pub enum PlaceChange {
    Name(#[validate] ApiEntityName),
    Description(#[validate] Option<ApiDescription>),
    Color(#[validate] ApiColor),
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/sensor/")]
pub struct PutPlace {
    pub place_name: ApiEntityName,
    #[validate]
    pub change: PlaceChange,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
pub enum GetPlaceEnum {
    FromPlaceName(#[validate] ApiEntityName),
    UserPlaces,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/place/")]
pub struct GetPlace {
    #[serde(flatten)]
    #[validate]
    pub param: GetPlaceEnum,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/place/")]
pub enum DeletePlace {
    FromPlaceName(#[validate] ApiEntityName),
    UserPlaces,
}

#[derive(TS, Debug, Serialize, Deserialize, Clone, Validate)]
#[ts(export, export_to = "./api/endpoints/place/")]
pub struct PostPlace {
    #[validate]
    pub name: ApiEntityName,
    #[validate]
    pub description: Option<ApiDescription>,
    #[validate]
    pub color: ApiColor,
}

pub struct Place {
    resources: Vec<Route>,
}

impl Endpoint for Place {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
    fn path(&self) -> &str {
        Self::API_PATH
    }
}

impl Place {
    pub const API_PATH: &str = "/place";
    pub fn new() -> Place {
        let mr = MethodRouter::new()
            .get(Self::place_get)
            .post(Self::place_post)
            .put(Self::place_put)
            .delete(Self::place_delete);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string(Self::API_PATH.to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn place_put(
        claims: Claims,
        mut conn: DbConnHolder,
        Query(PutPlace { place_name, change }): Query<PutPlace>,
    ) -> Result<Json<ApiUserPlace>, StatusCode> {
        let conn = &mut conn.0;
        let user_id =
            db::users::get_user(conn, db::users::Identifier::Username(&claims.username))?.id;
        let place = update_user_place(conn, change as Update, place_name.as_str(), user_id)?;
        let color = db::colors::get_color_by_id(conn, place.color_id)?;
        Ok(Json(ApiUserPlace::from_user_place_and_color(place, color)))
    }

    async fn place_get(
        claims: Claims,
        mut conn: DbConnHolder,
        Query(payload): Query<GetPlace>,
    ) -> Result<Json<Vec<ApiUserPlace>>, StatusCode> {
        let user_id = db::users::get_user(
            &mut conn.0,
            db::users::Identifier::Username(&claims.username),
        )?
        .id;

        let payload = payload.param;

        let id = match &payload {
            GetPlaceEnum::FromPlaceName(name) => {
                Identifier::PlaceNameAndUserId(name.as_str(), user_id)
            }
            GetPlaceEnum::UserPlaces => Identifier::UserId(user_id),
        };

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
                            name: up.name.into(),
                            description: up.description.map(|d| d.into()),
                            created_at: up.created_at.and_utc().timestamp() as usize,
                            updated_at: up.updated_at.and_utc().timestamp() as usize,
                            color: color.into(),
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

        log::trace!("place_get returning {} places", vec.len());

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

        log::trace!("Creating new place: {payload:?}");

        let color_id = db::colors::get_color_id(
            &mut conn.0,
            db::colors::Identifier::Hex(payload.color.clone().into()),
        )
        .inspect_err(|e| {
            log::warn!(
                "Error looking for ({color:?}): {e:?}",
                color = payload.color
            );
        })?;

        let place = NewUserPlace {
            user_id,
            name: payload.name.clone().into(),
            description: payload.description.map(|d| d.into()),
            color_id,
        };

        let res = db::user_places::insert_user_place(&mut conn.0, place)?;

        let res = ApiUserPlace {
            name: res.name.into(),
            description: res.description.map(|d| d.into()),
            color: payload.color,
            created_at: res.created_at.and_utc().timestamp() as usize,
            updated_at: res.updated_at.and_utc().timestamp() as usize,
        };

        log::trace!("Returning ApiUserPlace: {res:?}");

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

        log::trace!("Deleting place: {payload:?}");

        let id = match &payload {
            DeletePlace::FromPlaceName(name) => {
                Identifier::PlaceNameAndUserId(name.as_str(), user_id)
            }
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
                            name: up.name.into(),
                            description: up.description.map(|d| d.into()),
                            created_at: up.created_at.and_utc().timestamp() as usize,
                            updated_at: up.updated_at.and_utc().timestamp() as usize,
                            color: color.into(),
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

        log::trace!("Deleted {} places", vec.len());

        Ok(Json(vec))
    }
}

#[cfg(test)]
mod tests {

    use axum::extract::Query;
    use axum_serde_valid::Json;
    use serde_valid::json::ToJsonString;

    use crate::{
        api::endpoints::place::{DeletePlace, GetPlace, GetPlaceEnum, Place, PostPlace},
        auth::claims::{Claims, get_new_id},
        db::{
            DbConnHolder, establish_connection,
            tests::{create_test_user, create_test_user_place},
        },
    };

    #[tokio::test]
    async fn test_place_get_user_places() {
        let body = GetPlaceEnum::UserPlaces;

        let mut conn = establish_connection(true).unwrap();
        let (user, _) = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);

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
            Place::place_get(claims, DbConnHolder(conn), Query(GetPlace { param: body }))
                .await
                .expect("Should not fail");

        assert!(
            res_body.len() == 1,
            "res_body.len(): {}\nres_body: {:?}",
            res_body.len(),
            res_body.to_json_string()
        );
        assert_eq!(res_body.first().unwrap().name, user_place.name.into());
    }

    #[tokio::test]
    async fn test_place_get_api_id() {
        let mut conn = establish_connection(true).unwrap();
        let (user, _) = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);

        let body = GetPlaceEnum::FromPlaceName(user_place.name.clone().into());

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
            Place::place_get(claims, DbConnHolder(conn), Query(GetPlace { param: body }))
                .await
                .expect("Should not fail");

        assert!(
            res_body.len() == 1,
            "res_body.len(): {}\nres_body: {:?}",
            res_body.len(),
            res_body.to_json_string()
        );
        assert_eq!(res_body.first().unwrap().name, user_place.name.into());
    }

    #[tokio::test]
    async fn test_place_post() {
        let mut conn = establish_connection(true).unwrap();
        let (user, _) = create_test_user(&mut conn);

        let payload = PostPlace {
            name: "My New Awesome Place".to_string().into(),
            description: Some("A description for the new place.".to_string().into()),
            color: "#FF0000".to_string().into(),
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
        let res_body = Place::place_post(claims, DbConnHolder(conn), Json(payload.clone()))
            .await
            .expect("Should create a new place successfully");

        assert_eq!(res_body.name, payload.name);
        assert_eq!(res_body.description, payload.description);
        assert_eq!(res_body.color, payload.color);
    }

    #[tokio::test]
    async fn test_place_delete() {
        let mut conn = establish_connection(true).unwrap();
        let (user, _) = create_test_user(&mut conn);
        let place_to_delete = create_test_user_place(&mut conn, &user);

        let payload = DeletePlace::FromPlaceName(place_to_delete.name.clone().into());

        let claims = Claims {
            jwt_id: get_new_id(),
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
            place_to_delete.name.into()
        );
    }
}
