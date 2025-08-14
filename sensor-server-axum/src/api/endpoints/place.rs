use axum::{Json, routing::MethodRouter};
use chrono::NaiveDateTime;
use hyper::StatusCode;
use sensor_lib::api::model::api_id::ApiId;
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    middleware::extractor::jwt::Claims,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUserPlace {
    pub api_id: ApiId,
    pub user_api_id: ApiId,
    pub name: String,
    pub description: Option<String>,
    // pub color: Color, TODO: Create new color type on sensor-lib
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GetPlace {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PostPlace {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeletePlace {}

pub struct Place<'a> {
    resources: Vec<Route<'a>>,
}

impl<'a> Place<'a> {
    pub fn new() -> Place<'a> {
        let mr = MethodRouter::new()
            .get(Self::place_get)
            .post(Self::place_post)
            .delete(Self::place_delete);

        Self {
            resources: vec![Route::new(
                RoutePath::from_str("/places").expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn place_get(
        claims: Claims,
        Json(payload): Json<GetPlace>,
    ) -> (StatusCode, Json<Vec<ApiUserPlace>>) {
        todo!()
    }

    async fn place_post(
        claims: Claims,
        Json(payload): Json<PostPlace>,
    ) -> (StatusCode, Json<ApiUserPlace>) {
        todo!()
    }

    async fn place_delete(
        claims: Claims,
        Json(payload): Json<DeletePlace>,
    ) -> (StatusCode, Json<ApiUserPlace>) {
        todo!()
    }
}

impl<'a> Endpoint for Place<'a> {
    fn routes(&self) -> &[Route<'a>] {
        return &self.resources;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;

    use axum::Json;
    use hyper::StatusCode;
    use sensor_lib::api::model::api_id::ApiId;

    use crate::{
        api::endpoints::place::{GetPlace, Place},
        db::tests::{create_test_user, create_test_user_place, establish_test_connection},
        middleware::extractor::jwt::Claims,
    };

    #[tokio::test]
    async fn test_place_get() {
        let body = GetPlace {};

        let mut conn = establish_test_connection();
        let user = create_test_user(&mut conn);
        let user_place = create_test_user_place(&mut conn, &user);

        let claims = Claims {
            username: user.username,
            user_api_id: user.api_id,
            iat: chrono::Utc::now().timestamp(),
        };

        let (code, res_body) = Place::place_get(
            claims,
            Json::from_bytes(
                serde_json::to_string(&body)
                    .expect("Should be serializable")
                    .as_bytes(),
            )
            .expect("Json from Json"),
        )
        .await;

        assert_eq!(code, StatusCode::OK);
        assert!(res_body.len() == 1);
        assert_eq!(
            res_body.first().unwrap().api_id,
            ApiId::from_string(&user_place.api_id).expect("ApiId valid")
        );
    }
}
