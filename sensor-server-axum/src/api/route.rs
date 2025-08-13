use axum::routing::MethodRouter;

use crate::RoutePath;

pub struct Route<'a> {
    pub path: RoutePath<'a>,
    pub method_router: MethodRouter,
}

impl<'a> Route<'a> {
    pub fn new(path: RoutePath<'a>, method_router: MethodRouter) -> Route<'a> {
        Self {
            path,
            method_router,
        }
    }
}
