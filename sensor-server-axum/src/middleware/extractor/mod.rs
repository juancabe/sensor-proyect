use axum::{RequestPartsExt, extract::FromRequestParts};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use hyper::StatusCode;
use jsonwebtoken::{Validation, decode};

use crate::{
    auth::{claims::Claims, keys::KEYS},
    db::{DbConnHolder, establish_connection},
};

impl<S> FromRequestParts<S> for DbConnHolder
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        _parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        match establish_connection(false) {
            Ok(conn) => Ok(DbConnHolder(conn)),
            Err(e) => {
                log::error!(
                    "On call to establish_connection(true) when from_request_parts parsing for DbConnHolder: {e:?}"
                );
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(token_data.claims)
    }
}
