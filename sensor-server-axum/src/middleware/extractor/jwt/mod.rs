pub mod keys;

use axum::{
    Json, RequestPartsExt,
    extract::FromRequestParts,
    response::{IntoResponse, Response},
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use hyper::StatusCode;
use jsonwebtoken::{Validation, decode};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::middleware::extractor::jwt::keys::KEYS;

#[derive(Debug)]
pub enum Error {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Error::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            Error::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            Error::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            Error::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub username: String,
    pub user_api_id: String,
    pub iat: i64,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| Error::InvalidToken)?;

        // Decode the user data
        let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
            .map_err(|_| Error::InvalidToken)?;

        Ok(token_data.claims)
    }
}
