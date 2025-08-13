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
pub struct ApiUserSensor {
    pub api_id: String,
    pub place_api_id: String,
    pub device_id: String,
    pub name: String,
    pub description: Option<String>,
    // pub color: Color, TODO: Create new color type on sensor-lib
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GetSensor {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PostSensor {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeleteSensor {}

pub struct Sensor<'a> {
    resources: Vec<Route<'a>>,
}

impl<'a> Sensor<'a> {
    pub fn new() -> Sensor<'a> {
        let mr = MethodRouter::new()
            .get(Self::sensor_get)
            .post(Self::sensor_post)
            .delete(Self::sensor_delete);

        Self {
            resources: vec![Route::new(
                RoutePath::from_str("/sensors").expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn sensor_get(
        claims: Claims,
        Json(payload): Json<GetSensor>,
    ) -> (StatusCode, Json<ApiUserSensor>) {
        todo!()
    }

    async fn sensor_post(
        claims: Claims,
        Json(payload): Json<PostSensor>,
    ) -> (StatusCode, Json<ApiUserSensor>) {
        todo!()
    }

    async fn sensor_delete(
        claims: Claims,
        Json(payload): Json<DeleteSensor>,
    ) -> (StatusCode, Json<ApiUserSensor>) {
        todo!()
    }
}

impl<'a> Endpoint for Sensor<'a> {
    fn routes(&self) -> &[Route<'a>] {
        return &self.resources;
    }
}
