use axum::routing::MethodRouter;
use hyper::StatusCode;

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
};

pub struct Health {
    resources: Vec<Route>,
}

impl Endpoint for Health {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
}

impl Health {
    pub fn new() -> Health {
        let mr = MethodRouter::new().get(Self::health_get);

        Self {
            resources: vec![Route::new(
                RoutePath::from_string("/health".to_string()).expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn health_get() -> (StatusCode, String) {
        (StatusCode::OK, "OK".into())
    }
}
