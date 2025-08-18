use axum::{Router, routing::MethodRouter};

use crate::{
    api::{Endpoint, endpoints::generate_endpoints},
    auth::keys::KEYS,
    db::establish_connection,
};

pub type ServerMethodRouter = MethodRouter;

#[derive(Debug, Clone)]
pub struct State {}

impl State {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct SensorServer {
    endpoints: Vec<Box<dyn Endpoint>>,
    state: State,
}

impl SensorServer {
    pub fn new() -> Self {
        // Load LazyStatics
        let _ = *KEYS;
        log::info!("Loaded keys for JWT");
        establish_connection().expect("Connection should be available");
        log::info!("Loaded DB_POOL");

        let endpoints = generate_endpoints();

        let state = State::new();

        Self { endpoints, state }
    }

    pub fn routes(&self) -> impl Iterator<Item = (String, ServerMethodRouter)> {
        self.endpoints
            .iter()
            .map(|endpoint| endpoint.routes())
            .flatten()
            .map(|route| {
                (
                    String::from("/api/v0") + route.path.as_str(),
                    route.method_router.clone(),
                )
            })
    }

    pub fn into_router(self) -> Router {
        let mut router = Router::new();

        for (path, route) in self.routes() {
            router = router.route(&path, route);
        }

        router
        // .with_state(self.state)
    }
}

#[cfg(test)]
mod tests {
    use axum_test::TestServer;

    use crate::sensor_server::SensorServer;

    #[test]
    fn test_sensor_server() {
        let sensor_server = SensorServer::new();
    }

    #[tokio::test]
    #[ignore = "DB should not include test in name, must commit changes and then be reverted"]
    async fn test_integration() {
        // This test should be run in a db that can be 'migration redo'

        let server = TestServer::new(SensorServer::new().into_router())
            .expect("Should be created successfully");

        let res = server.get("/api/v0/health").await;
        res.assert_status_ok();
        res.assert_text("OK"); // or whatever your health endpoint returns
    }

    // #[tokio::test]
    // async fn healthcheck_works() {
    //     let server = test_server();
    //
    //     // Assuming one of your endpoints exposes GET /api/v0/health
    //     let res = server.get("/api/v0/health").await;
    //     res.assert_status_ok();
    //     res.assert_text("OK"); // or whatever your health endpoint returns
    // }
}
