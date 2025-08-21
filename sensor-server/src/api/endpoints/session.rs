use axum::{extract::Query, routing::MethodRouter};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use axum_serde_valid::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use time::Duration;
use ts_rs::TS;

use crate::{
    RoutePath,
    api::{
        Endpoint,
        route::Route,
        types::validate::{api_raw_password::ApiRawPassword, api_username::ApiUsername},
    },
    auth::claims::Claims,
    db::{self, DbConnHolder, users},
    state::PoisonableIdentifier,
};

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/session/")]
pub struct GetSession {
    #[validate]
    pub username: ApiUsername,
    #[validate]
    pub raw_password: ApiRawPassword,
}

#[derive(TS, Debug, Serialize, Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/session/")]
pub struct PostSession {}

#[derive(TS, Debug, Serialize, Deserialize, Clone)]
#[ts(export, export_to = "./api/endpoints/session/")]
// WARN: Dont accept this in any endpoint
// WARN Every time this struct is returned, the response MUST return a Set-Cookie with the JWT
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

    pub fn from_claims(claims: Claims) -> Result<Self, jsonwebtoken::errors::Error> {
        let jwt = claims.encode_jwt()?;
        Ok(Self::new(jwt, claims.exp - claims.iat))
    }

    pub fn build_cookie<'a, 'b>(&'a self) -> Cookie<'b> {
        Cookie::build(("access_token", self.access_token.clone()))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(Duration::seconds(self.expires_in as i64))
            .into()
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
        jar: CookieJar,
        claims: Claims,
    ) -> Result<(CookieJar, Json<ApiSession>), StatusCode> {
        log::trace!("Renewing JWT for user: {}", claims.username);
        // Poison outdated JWT
        PoisonableIdentifier::JWTId(claims.jwt_id_hex()).poison()?;

        let username = claims.username.clone();

        let claims = Claims::new(claims.username);

        match ApiSession::from_claims(claims) {
            Ok(ass) => Ok((jar.add(ass.build_cookie()), Json(ass))),
            Err(e) => {
                log::error!(
                    "Error generating new session from_claims for username ({username}): {e:?}"
                );
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn session_get(
        mut conn: DbConnHolder,
        jar: CookieJar,
        Query(payload): Query<GetSession>,
    ) -> Result<(CookieJar, Json<ApiSession>), StatusCode> {
        log::trace!("Generating new JWT");

        let db_hashed_password = users::get_user(
            &mut conn.0,
            users::Identifier::Username(payload.username.as_str()),
        )
        .map_err(|e| match e {
            db::Error::NotFound(error) => {
                log::warn!(
                    "Tried to login to user: {username:?} but it doesn't exist: {error:?}",
                    username = &payload.username
                );
                StatusCode::UNAUTHORIZED
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .hashed_password;

        if !payload
            .raw_password
            .password_matches_raw(&db_hashed_password)
        {
            log::warn!("Passwords didn't match for payload: {payload:?}");
            return Err(StatusCode::UNAUTHORIZED);
        }

        if PoisonableIdentifier::Username(payload.username.clone().into()).is_poisoned()? {
            log::error!(
                "Username should not be in DB when its poisoned, username: {:?}",
                payload.username
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)?
        }

        let claims = Claims::new(payload.username.into());

        match ApiSession::from_claims(claims) {
            Ok(ass) => Ok((jar.add(ass.build_cookie()), Json(ass))),
            Err(e) => {
                log::error!("Error generating new claims: {e:?}");
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
        auth::claims::{Claims, get_new_id},
        db::{establish_connection, tests::create_test_user},
    };

    use super::*;

    #[tokio::test]
    async fn test_session_post() {
        let mut conn_nref = establish_connection(true).unwrap();
        let conn = &mut conn_nref;

        let (user, _) = create_test_user(conn);

        let now = chrono::Utc::now();
        let half_day = TimeDelta::hours(12);

        let claims = Claims {
            jwt_id: get_new_id(),
            username: user.username,
            iat: user.updated_auth_at.and_utc().timestamp() as usize - 1,
            exp: now.checked_add_signed(half_day).unwrap().timestamp() as usize,
        };

        Session::session_post(CookieJar::new(), claims)
            .await
            .expect("Should not fail");
    }

    #[tokio::test]
    async fn test_session_get() {
        let mut conn_nref = establish_connection(true).unwrap();
        let conn = &mut conn_nref;
        let (user, pass) = create_test_user(conn);

        let json = GetSession {
            username: user.username.into(),
            raw_password: pass,
        };

        let conn = DbConnHolder(conn_nref);
        let _res = Session::session_get(conn, CookieJar::new(), Query(json))
            .await
            .expect("Should not fail");
    }
}
