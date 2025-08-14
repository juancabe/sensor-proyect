use axum::routing::MethodRouter;

use crate::RoutePath;

pub struct Route {
    pub path: RoutePath,
    pub method_router: MethodRouter,
}

impl Route {
    pub fn new(path: RoutePath, method_router: MethodRouter) -> Route {
        Self {
            path,
            method_router,
        }
    }
}
