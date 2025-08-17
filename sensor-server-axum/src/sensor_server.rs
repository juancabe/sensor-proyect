use axum::routing::MethodRouter;

use crate::{
    api::{Endpoint, endpoints::generate_endpoints},
    auth::keys::KEYS,
    db::establish_connection,
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
        establish_connection().expect("Connection should be available");
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
                    String::from("/api/v0") + route.path.as_str(),
                    &route.method_router,
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use axum::Router;
    use axum_test::TestServer;

    use crate::sensor_server::SensorServer;

    fn load_router(sensor_server: SensorServer) -> Router {
        let mut router = Router::new();

        for (path, method_router) in sensor_server.routes() {
            router = router.route(&path, method_router.clone());
        }

        router
    }

    #[test]
    fn test_sensor_server() {
        let sensor_server = SensorServer::new();
        let _ = load_router(sensor_server);
    }

    // A test-oriented constructor to avoid real side-effects
    fn test_server() -> TestServer {
        let sensor_server = SensorServer::new();
        let router = load_router(sensor_server);
        TestServer::new(router).expect("start test server")
    }

    #[tokio::test]
    async fn healthcheck_works() {
        let server = test_server();

        // Assuming one of your endpoints exposes GET /api/v0/health
        let res = server.get("/api/v0/health").await;
        res.assert_status_ok();
        res.assert_text("OK"); // or whatever your health endpoint returns
    }
}
