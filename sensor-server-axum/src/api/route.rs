use crate::{RoutePath, sensor_server::ServerMethodRouter};

pub struct Route {
    pub path: RoutePath,
    pub method_router: ServerMethodRouter,
}

impl Route {
    pub fn new(path: RoutePath, method_router: ServerMethodRouter) -> Route {
        Self {
            path,
            method_router,
        }
    }
}
