use axum::routing::MethodRouter;

use crate::{
    api::{Endpoint, endpoints::generate_endpoints},
    db::DB_POOL,
    middleware::extractor::jwt::keys::KEYS,
};

pub struct SensorServer {
    endpoints: Vec<Box<dyn Endpoint>>,
    // pub routes: Vec<(&'a str, &'a MethodRouter)>,
}

impl SensorServer {
    pub fn new() -> Self {
        // Load LazyStatics
        let _ = *KEYS;
        log::info!("Loaded keys for JWT");
        let _ = *DB_POOL;
        log::info!("Loaded DB_POOL");

        let endpoints = generate_endpoints();

        Self { endpoints }
    }

    pub fn routes(&self) -> impl Iterator<Item = (String, &MethodRouter)> {
        self.endpoints
            .iter()
            .map(|endpoint| endpoint.routes())
            .flatten()
            .map(|route| {
                (
                    String::from("/api/v0/") + route.path.as_str(),
                    &route.method_router,
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::sensor_server::SensorServer;

    #[test]
    fn test_sensor_server() {
        SensorServer::new();
    }
}
