use axum::{Json, routing::MethodRouter};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSession {}

pub struct Session<'a> {
    resources: Vec<Route<'a>>,
}

impl<'a> Session<'a> {
    pub fn new() -> Session<'a> {
        let mr = MethodRouter::new();
        Self {
            resources: vec![Route::new(
                RoutePath::from_str("/session").expect("The route should be correct"),
                mr,
            )],
        }
    }
}

impl<'a> Endpoint for Session<'a> {
    fn routes(&self) -> &[Route<'a>] {
        return &self.resources;
    }
}
