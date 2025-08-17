use axum::{Json, routing::MethodRouter};
use chrono::NaiveDateTime;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    auth::claims::Claims,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUser {
    pub username: String,
    pub api_id: String,
    pub hashed_password: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GetUser {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PostUser {}

pub struct User {
    resources: Vec<Route>,
}

impl User {
    pub fn new() -> User {
        let mr = MethodRouter::new()
            .get(Self::user_get)
            .post(Self::user_post);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string("/user".to_string()).expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn user_get(claims: Claims, Json(payload): Json<GetUser>) -> (StatusCode, Json<ApiUser>) {
        todo!()
    }

    async fn user_post(
        claims: Claims,
        Json(payload): Json<PostUser>,
    ) -> (StatusCode, Json<ApiUser>) {
        todo!()
    }
}

impl Endpoint for User {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
}
