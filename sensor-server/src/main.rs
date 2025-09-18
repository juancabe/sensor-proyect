use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use axum_server::tls_rustls::RustlsConfig;
use dotenv::dotenv;
use sensor_server::{PORT, sensor_server::SensorServer};

#[cfg(not(feature = "production"))]
const CERTS_DIR: &str = "self_signed_certs";
#[cfg(feature = "production")]
const CERTS_DIR: &str = "authorized_certs";

#[tokio::main]
async fn main() {
    dotenv().expect(".env should exist and be readable");
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Trace)
        .parse_default_env()
        .init();

    let sensor_server = SensorServer::new();

    let config = RustlsConfig::from_pem_file(
        PathBuf::from("./").join(CERTS_DIR).join("cert.pem"),
        PathBuf::from("./").join(CERTS_DIR).join("key.pem"),
    )
    .await
    .unwrap();

    let router = sensor_server.into_router();

    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, PORT));
    axum_server::bind_rustls(addr, config)
        .serve(router.into_make_service())
        .await
        .unwrap()
}
