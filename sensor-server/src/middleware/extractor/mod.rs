use axum::extract::FromRequestParts;
use axum_extra::{
    TypedHeader,
    extract::CookieJar,
    headers::{Authorization, authorization::Bearer},
};
use hyper::StatusCode;
use jsonwebtoken::{Validation, decode};

use crate::{
    auth::{claims::Claims, keys::KEYS},
    db::{DbConnHolder, establish_connection},
    state,
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
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Ok(TypedHeader(Authorization(bearer))) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
        {
            log::trace!("JWT found in headers: {bearer:?}");
            return verify(bearer.token());
        }

        let Ok(jar) = CookieJar::from_request_parts(parts, state).await;
        if let Some(cookie) = jar.get("access_token") {
            log::trace!("JWT found in cookie: {cookie:?}");
            return verify(cookie.value());
        }

        log::warn!("No Header nor cookie found with available JWT, UNAUTHORIZED");
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// common logic to decode + poison-check
fn verify(jwt: &str) -> Result<Claims, StatusCode> {
    let token_data =
        decode::<Claims>(jwt, &KEYS.decoding, &Validation::default()).map_err(|e| {
            log::warn!("Unable to decode JWT: {e:?}");
            StatusCode::UNAUTHORIZED
        })?;

    // Check state poisoned status
    if state::PoisonableIdentifier::JWTId(token_data.claims.jwt_id_hex()).is_poisoned()? {
        log::warn!("Tried to access with poisoned JWT: {jwt}, token_data: {token_data:?}");
        return Err(StatusCode::UNAUTHORIZED);
    }
    if state::PoisonableIdentifier::Username(token_data.claims.username.clone()).is_poisoned()? {
        log::warn!(
            "Tried to access with poisoned username, JWT: {jwt}, token_data: {token_data:?}"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(token_data.claims)
}
