use axum::extract::FromRequestParts;
use hyper::StatusCode;

use crate::db::{DbConn, establish_connection};

pub mod jwt;

#[derive()]
pub struct DbConnHolder(pub DbConn);
impl<S> FromRequestParts<S> for DbConnHolder
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        match establish_connection() {
            Ok(conn) => Ok(DbConnHolder(conn)),
            Err(e) => {
                log::error!(
                    "On call to establish_connection when from_request_parts parsing for DbConnHolder: {e:?}"
                );
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}
