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
pub struct ApiSensorData {
    pub sensor_api_id: String,
    pub data: serde_json::value::Value,
    pub added_at: NaiveDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct GetSensorData {}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct PostSensorData {}

pub struct SensorData<'a> {
    resources: Vec<Route<'a>>,
}

impl<'a> SensorData<'a> {
    pub fn new() -> SensorData<'a> {
        let mr = MethodRouter::new()
            .get(Self::sensor_get)
            .post(Self::sensor_post);

        Self {
            resources: vec![Route::new(
                RoutePath::from_str("/sensor_data").expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn sensor_get(
        claims: Claims,
        Json(payload): Json<GetSensorData>,
    ) -> (StatusCode, Json<ApiSensorData>) {
        todo!()
    }

    async fn sensor_post(
        claims: Claims,
        Json(payload): Json<PostSensorData>,
    ) -> (StatusCode, Json<ApiSensorData>) {
        todo!()
    }
}

impl<'a> Endpoint for SensorData<'a> {
    fn routes(&self) -> &[Route<'a>] {
        return &self.resources;
    }
}
