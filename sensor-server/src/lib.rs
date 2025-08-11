use std::net::SocketAddr;
use std::{io, sync::Arc};

use std::fs;

pub mod db;
pub mod helper;
pub mod models;
pub mod schema;
pub mod server;

use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;

use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::db::test_db_pool;

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    // Open certificate file.
    let certfile =
        fs::File::open(filename).map_err(|e| error(format!("failed to open {filename}: {e}")))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    rustls_pemfile::certs(&mut reader).collect()
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
    // Open keyfile.
    let keyfile =
        fs::File::open(filename).map_err(|e| error(format!("failed to open {filename}: {e}")))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap())
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Sets default log level to Debug
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    log::info!("Starting sensor server...");
    test_db_pool().unwrap();

    // Load public cert
    let cert = load_certs("private/fullchain.pem")?;
    // Load private key
    let key = load_private_key("private/privkey.pem")?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    // We create a TcpListener and bind it to 0.0.0.0:3000
    let listener = TcpListener::bind(addr).await?;

    let mut server_cnfg = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)?;
    server_cnfg.alpn_protocols = vec![b"http/1.1".to_vec(), b"http/1.0".to_vec()];

    let tls_acceptor = TlsAcceptor::from(Arc::new(server_cnfg));

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;
        log::info!("Accepted connection from {}", stream.peer_addr()?);

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let tls_acceptor = tls_acceptor.clone();

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            let tls_stream = match tls_acceptor.accept(stream).await {
                Ok(tls_stream) => tls_stream,
                Err(e) => {
                    eprintln!("failed to perform tls handshake: {e:#}");
                    return;
                }
            };

            let io = TokioIo::new(tls_stream);

            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(server::server))
                .await
            {
                log::error!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[cfg(test)]
mod tests {}
