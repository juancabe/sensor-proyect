use axum::{Json, routing::MethodRouter};
use hyper::StatusCode;
use jsonwebtoken::{Header, encode};
use serde::{Deserialize, Serialize};

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    auth::{claims::Claims, keys::KEYS},
    db::{self, DbConnHolder, users},
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

    async fn session_post(
        mut conn: DbConnHolder,
        claims: Claims,
    ) -> Result<Json<ApiSession>, StatusCode> {
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
            Err(_) => todo!(),
        }
    }

    pub async fn session_get(
        mut conn: DbConnHolder,
        Json(payload): Json<GetSession>,
    ) -> Result<Json<ApiSession>, StatusCode> {
        let db_hashed_password =
            users::get_user(&mut conn.0, users::Identifier::Username(&payload.username))?
                .hashed_password;

        if db_hashed_password != payload.hashed_password {
            return Err(StatusCode::UNAUTHORIZED);
        }

        let claims = Claims::new(payload.username);

        match encode(&Header::default(), &claims, &KEYS.encoding) {
            Ok(jwt) => Ok(Json(ApiSession::new(jwt, claims.exp - claims.iat))),
            Err(_) => todo!(),
        }
    }
}

impl Endpoint for Session {
    fn routes(&self) -> &[Route] {
        return &self.resources;
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
        let mut conn_nref = establish_connection().unwrap();
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
        let mut conn_nref = establish_connection().unwrap();
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
        let mut conn_nref = establish_connection().unwrap();
        let conn = &mut conn_nref;
        let user = create_test_user(conn);

        let json = GetSession {
            username: user.username,
            hashed_password: user.hashed_password,
        };

        let conn = DbConnHolder(conn_nref);
        let _res = Session::session_get(conn, Json(json))
            .await
            .expect("Should not fail");
    }
}
