use axum::extract::FromRequestParts;
use axum_extra::{
    TypedHeader,
    extract::CookieJar,
    headers::{Authorization, authorization::Bearer},
};
use hyper::StatusCode;

use crate::{
    auth::{claims::Claims, sensor_claims::SensorClaims},
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
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Ok(TypedHeader(Authorization(bearer))) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await
        {
            log::trace!("JWT found in headers: {bearer:?}");
            return Claims::from_jwt(bearer.token());
        }

        let Ok(jar) = CookieJar::from_request_parts(parts, state).await;
        if let Some(cookie) = jar.get("access_token") {
            log::trace!("JWT found in cookie: {cookie:?}");
            return Claims::from_jwt(cookie.value());
        }

        log::warn!("No Header nor cookie found with available JWT, UNAUTHORIZED");
        Err(StatusCode::UNAUTHORIZED)
    }
}

impl<S> FromRequestParts<S> for SensorClaims
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
            return SensorClaims::from_jwt(bearer.token());
        }

        let Ok(jar) = CookieJar::from_request_parts(parts, state).await;
        if let Some(cookie) = jar.get("access_token") {
            log::trace!("JWT found in cookie: {cookie:?}");
            return SensorClaims::from_jwt(cookie.value());
        }

        log::warn!("No Header nor cookie found with available JWT, UNAUTHORIZED");
        Err(StatusCode::UNAUTHORIZED)
    }
}
