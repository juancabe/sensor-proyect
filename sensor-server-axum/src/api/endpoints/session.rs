use axum::{Json, extract::Query, routing::MethodRouter};
use hyper::StatusCode;
use jsonwebtoken::{Header, encode};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    auth::{claims::Claims, keys::KEYS},
    db::{self, DbConnHolder, users},
};

#[derive(TS, Debug, Serialize, Deserialize)]
#[ts(export, export_to = "./api/endpoints/session/")]
pub struct GetSession {
    pub username: String,
    pub hashed_password: String,
}

#[derive(TS, Debug, Serialize, Deserialize)]
#[ts(export, export_to = "./api/endpoints/session/")]
pub struct PostSession {}

#[derive(TS, Debug, Serialize, Deserialize)]
#[ts(export, export_to = "./api/endpoints/session/")]
pub struct ApiSession {
    pub access_token: String,
    pub expires_in: usize,
    token_type: String,
}

impl ApiSession {
    fn new(access_token: String, expires_in: usize) -> Self {
        Self {
            access_token,
            expires_in,
            token_type: "Bearer".to_string(),
        }
    }
}

pub struct Session {
    resources: Vec<Route>,
}

impl Session {
    pub const API_PATH: &str = "/session";
    pub fn new() -> Session {
        let mr = MethodRouter::new()
            .get(Self::session_get)
            .post(Self::session_post);
        Self {
            resources: vec![Route::new(
                RoutePath::from_string(Self::API_PATH.to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn session_post(
        mut conn: DbConnHolder,
        claims: Claims,
    ) -> Result<Json<ApiSession>, StatusCode> {
        log::trace!("Renewing JWT for user: {}", claims.username);
        {
            // Check if the user changed auth between JWT renewals
            let user = db::users::get_user(
                &mut conn.0,
                db::users::Identifier::Username(&claims.username),
            )
            .map_err(|e| match e {
                db::Error::NotFound(e) => {
                    log::info!("The user didn't exist so the DB returned the code: {e:?}");
                    StatusCode::UNAUTHORIZED
                }
                _ => e.into(),
            })?;

            if user.updated_auth_at.and_utc().timestamp() as usize > claims.iat {
                log::warn!(
                    "User {} tried to session_post when his user.updated_auth_at > claims.iat",
                    user.username
                );
                Err(StatusCode::UNAUTHORIZED)?
            }
        }

        let claims = Claims::new(claims.username);

        match encode(&Header::default(), &claims, &KEYS.encoding) {
            Ok(jwt) => Ok(Json(ApiSession::new(jwt, claims.exp - claims.exp))),
            Err(e) => {
                log::error!(
                    "Error generating new claims for encode(default, {claims:?}, &KEYS.encoding): {e:?}"
                );
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn session_get(
        mut conn: DbConnHolder,
        Query(payload): Query<GetSession>,
    ) -> Result<Json<ApiSession>, StatusCode> {
        log::trace!("Generating new JWT");

        let db_hashed_password =
            users::get_user(&mut conn.0, users::Identifier::Username(&payload.username))
                .map_err(|e| match e {
                    db::Error::NotFound(error) => {
                        log::warn!(
                            "Tried to login to user: {username} but it doesn't exist: {error:?}",
                            username = &payload.username
                        );
                        StatusCode::UNAUTHORIZED
                    }
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                })?
                .hashed_password;

        if db_hashed_password != payload.hashed_password {
            log::warn!("Passwords didn't match for payload: {payload:?}");
            return Err(StatusCode::UNAUTHORIZED);
        }

        let claims = Claims::new(payload.username);

        match encode(&Header::default(), &claims, &KEYS.encoding) {
            Ok(jwt) => Ok(Json(ApiSession::new(jwt, claims.exp - claims.iat))),
            Err(e) => {
                log::error!(
                    "Error generating new claims for encode(default, {claims:?}, &KEYS.encoding): {e:?}"
                );
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

impl Endpoint for Session {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }

    fn path(&self) -> &str {
        Self::API_PATH
    }
}

#[cfg(test)]
mod test {

    use chrono::TimeDelta;

    use crate::{
        auth::claims::Claims,
        db::{establish_connection, tests::create_test_user},
    };

    use super::*;

    #[tokio::test]
    async fn test_session_post() {
        let mut conn_nref = establish_connection(true).unwrap();
        let conn = &mut conn_nref;

        let user = create_test_user(conn);

        let now = chrono::Utc::now();
        let half_day = TimeDelta::hours(12);

        let claims = Claims {
            username: user.username,
            iat: user.updated_auth_at.and_utc().timestamp() as usize - 1, // iat should be bigger
            exp: now.checked_add_signed(half_day).unwrap().timestamp() as usize,
        };

        let conn = DbConnHolder(conn_nref);

        match Session::session_post(conn, claims).await {
            Ok(_) => {
                panic!("Should have failed");
            }
            Err(_) => (),
        }
    }

    #[tokio::test]
    async fn test_session_post_should_fail() {
        let mut conn_nref = establish_connection(true).unwrap();
        let conn = &mut conn_nref;

        let user = create_test_user(conn);

        let now = chrono::Utc::now();
        let half_day = TimeDelta::hours(12);

        let claims = Claims {
            username: user.username,
            iat: user.updated_auth_at.and_utc().timestamp() as usize, // iat should be bigger
            exp: now.checked_add_signed(half_day).unwrap().timestamp() as usize,
        };

        let conn = DbConnHolder(conn_nref);

        let _session = Session::session_post(conn, claims)
            .await
            .expect("Should not fail");
    }

    #[tokio::test]
    async fn test_session_get() {
        let mut conn_nref = establish_connection(true).unwrap();
        let conn = &mut conn_nref;
        let user = create_test_user(conn);

        let json = GetSession {
            username: user.username,
            hashed_password: user.hashed_password,
        };

        let conn = DbConnHolder(conn_nref);
        let _res = Session::session_get(conn, Query(json))
            .await
            .expect("Should not fail");
    }
}
