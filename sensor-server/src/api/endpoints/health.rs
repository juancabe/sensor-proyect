use axum::{extract::Query, routing::MethodRouter};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
};

#[derive(Serialize, Deserialize, TS)]
#[ts(export, export_to = "./api/endpoints/health/")]
pub struct GetHealth {}

pub struct Health {
    resources: Vec<Route>,
}

impl Endpoint for Health {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }

    fn path(&self) -> &str {
        Self::API_PATH
    }
}

impl Health {
    pub const API_PATH: &str = "/health";

    pub fn new() -> Health {
        let mr = MethodRouter::new().get(Self::health_get);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string(Self::API_PATH.to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn health_get(Query(_): Query<GetHealth>) -> (StatusCode, String) {
        (StatusCode::OK, "OK".into())
    }
}
