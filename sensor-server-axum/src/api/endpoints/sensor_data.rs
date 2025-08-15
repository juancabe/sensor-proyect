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

pub struct SensorData {
    resources: Vec<Route>,
}

impl SensorData {
    pub fn new() -> SensorData {
        let mr = MethodRouter::new()
            .get(Self::sensor_data_get)
            .post(Self::sensor_data_post);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string("/sensor_data".to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn sensor_data_get(
        claims: Claims,
        Json(payload): Json<GetSensorData>,
    ) -> (StatusCode, Json<ApiSensorData>) {
        todo!()
    }

    async fn sensor_data_post(
        claims: Claims,
        Json(payload): Json<PostSensorData>,
    ) -> (StatusCode, Json<ApiSensorData>) {
        todo!()
    }
}

impl Endpoint for SensorData {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
}
