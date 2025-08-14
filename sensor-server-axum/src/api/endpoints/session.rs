use axum::{Json, routing::MethodRouter};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSession {}

pub struct Session {
    resources: Vec<Route>,
}

impl Session {
    pub fn new() -> Session {
        let mr = MethodRouter::new();
        Self {
            resources: vec![Route::new(
                RoutePath::from_string("/session".to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }
}

impl Endpoint for Session {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
}
