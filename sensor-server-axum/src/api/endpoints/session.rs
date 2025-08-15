use axum::{Json, routing::MethodRouter};
use chrono::TimeDelta;
use hyper::StatusCode;
use jsonwebtoken::{Header, encode};
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    db::users,
    middleware::extractor::{
        DbConnHolder,
        jwt::{Claims, keys::KEYS},
    },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct GetSession {
    pub username: String,
    pub hashed_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSession {
    pub access_token: String,
    pub expires_in: usize,
    token_type: &'static str,
}

impl ApiSession {
    fn new(access_token: String, expires_in: usize) -> Self {
        Self {
            access_token,
            expires_in,
            token_type: "Bearer",
        }
    }
}

pub struct Session {
    resources: Vec<Route>,
}

impl Session {
    pub fn new() -> Session {
        let mr = MethodRouter::new()
            .get(Self::session_get)
            .post(Self::session_post);
        Self {
            resources: vec![Route::new(
                RoutePath::from_string("/session".to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn session_post(claims: Claims) -> Result<Json<ApiSession>, StatusCode> {
        let now = chrono::Utc::now();
        let expires_in = TimeDelta::days(1);
        let tomorrow = now
            .checked_add_signed(expires_in)
            .expect("Should not be out of range");

        let claims = Claims {
            username: claims.username,
            iat: now.timestamp() as usize,
            exp: tomorrow.timestamp() as usize,
        };

        match encode(&Header::default(), &claims, &KEYS.encoding) {
            Ok(jwt) => Ok(Json(ApiSession::new(
                jwt,
                expires_in.num_seconds() as usize,
            ))),
            Err(_) => todo!(),
        }
    }

    async fn session_get(
        mut conn: DbConnHolder,
        Json(payload): Json<GetSession>,
    ) -> Result<Json<ApiSession>, StatusCode> {
        let db_hashed_password = users::get_user_password(
            &mut conn.0,
            users::Identifier::Username(payload.username.clone()),
        )?;

        if db_hashed_password != payload.hashed_password {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let now = chrono::Utc::now();
        let expires_in = TimeDelta::days(1);
        let tomorrow = now
            .checked_add_signed(expires_in)
            .expect("Should not be out of range");

        let claims = Claims {
            username: payload.username,
            iat: now.timestamp() as usize,
            exp: tomorrow.timestamp() as usize,
        };

        match encode(&Header::default(), &claims, &KEYS.encoding) {
            Ok(jwt) => Ok(Json(ApiSession::new(
                jwt,
                expires_in.num_seconds() as usize,
            ))),
            Err(_) => todo!(),
        }
    }
}

impl Endpoint for Session {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
}
