use std::net::SocketAddr;

use axum::{
    body::{self, Body},
    extract::Request,
    middleware::Next,
    response::Response,
};
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

    log::info!("Request started\nip={ip}, auth_header={auth_header:?}, method={method}, uri={uri}",);

    // Split and read the body
    // WARN: This operation is heavy and unsafe (max body = 3MB), remove for prod
    let (parts, body) = req.into_parts();
    let bytes = body::to_bytes(body, 3_000_000).await.map_err(|e| {
        log::error!("Error extracting body for printing: {e:?}");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    log::debug!("body: {}", String::from_utf8_lossy(bytes.iter().as_slice()));
    let body = Body::from(bytes);
    let req = Request::from_parts(parts, body);

    // Run real handler -- Reconstruct req
    let res = next.run(req).await;

    log::info!(
        "Request finished\nip={ip}, auth_header={auth_header:?}, method={method}, uri={uri}, res={res}",
        res = res.status()
    );

    Ok(res)
}
