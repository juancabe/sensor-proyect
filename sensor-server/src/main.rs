use std::net::{Ipv4Addr, SocketAddrV4};

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

    let router = sensor_server.into_router();

    axum::serve(listener, router).await.unwrap()
}
