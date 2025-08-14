use std::net::{Ipv4Addr, SocketAddrV4};

use axum::Router;
use dotenv::dotenv;
use sensor_server_axum::{PORT, sensor_server::SensorServer};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().expect(".env should exist and be readable");
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Trace)
        .parse_default_env()
        .init();

    let sensor_server = SensorServer::new();

    let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT))
        .await
        .unwrap();
    let mut router = Router::new();

    for (path, method_router) in sensor_server.routes() {
        router = router.route(&path, method_router.clone()); // TODO: Get rid of clone
    }

    axum::serve(listener, router).await.unwrap()
}
