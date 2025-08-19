use std::net::SocketAddr;

use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use hyper::StatusCode;

pub mod extractor;

pub async fn log_request(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    let ip = req
        .extensions()
        .get::<SocketAddr>() // set by `into_make_service_with_connect_info`
        .map(SocketAddr::ip)
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| "<unknown>".into());

    let method = req.method().clone();
    let uri = req.uri().clone();
    let auth_header = req.headers().clone();
    let auth_header = auth_header.get("Authorization");

    log::info!("Request started\nip={ip}, auth_header={auth_header:?}, method={method}, uri={uri}");
    // Run real handler
    let res = next.run(req).await;

    log::info!(
        "Request finished\nip={ip}, auth_header={auth_header:?}, method={method}, uri={uri}, res={res}",
        res = res.status()
    );

    Ok(res)
}
