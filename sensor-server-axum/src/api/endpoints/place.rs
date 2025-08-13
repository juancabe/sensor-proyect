use axum::{Json, routing::MethodRouter};
use chrono::NaiveDateTime;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    middleware::extractor::jwt::Claims,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiUserPlace {
    pub api_id: String,
    pub user_api_id: String,
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
    ) -> (StatusCode, Json<ApiUserPlace>) {
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
