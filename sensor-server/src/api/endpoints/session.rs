use axum::routing::MethodRouter;
use axum_extra::extract::{CookieJar, cookie::Cookie};
use axum_serde_valid::Json;
use common::endpoints_io::session::{ApiSession, PostSession};
use hyper::StatusCode;
use time::Duration;

use crate::{
    RoutePath,
    api::{Endpoint, route::Route},
    auth::{claims::Claims, sensor_claims::SensorClaims},
    db::{self, DbConnHolder, user_sensors::AuthorizedSensor, users},
    state::poisonable_identifier::PoisonableIdentifier,
};

#[derive(Debug)]
pub struct ServerApiSession(ApiSession);
impl ServerApiSession {
    fn new(access_token: String, expires_in: usize) -> Self {
        Self(ApiSession::new(access_token, expires_in))
    }

    pub fn from_claims(claims: Claims) -> Result<Self, jsonwebtoken::errors::Error> {
        let jwt = claims.encode_jwt()?;
        Ok(Self::new(jwt, claims.exp - claims.iat))
    }

    pub fn from_sensor_claims(claims: SensorClaims) -> Result<Self, jsonwebtoken::errors::Error> {
        let jwt = claims.encode_jwt()?;
        Ok(Self::new(jwt, claims.exp - claims.iat))
    }

    pub fn build_cookie<'a, 'b>(&'a self) -> Cookie<'b> {
        Cookie::build(("access_token", self.0.access_token.clone()))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .max_age(Duration::seconds(self.0.expires_in as i64))
            .into()
    }
}

impl From<ApiSession> for ServerApiSession {
    fn from(value: ApiSession) -> Self {
        Self(value)
    }
}

impl Into<ApiSession> for ServerApiSession {
    fn into(self) -> ApiSession {
        self.0
    }
}

pub struct Session {
    resources: Vec<Route>,
}

impl Session {
    pub const API_PATH: &str = "/session";
    pub fn new() -> Session {
        let mr = MethodRouter::new()
            .post(Self::session_post)
            .put(Self::session_put);
        Self {
            resources: vec![Route::new(
                RoutePath::from_string(Self::API_PATH.to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn session_put(
        jar: CookieJar,
        claims: Claims,
    ) -> Result<(CookieJar, Json<ApiSession>), StatusCode> {
        log::trace!("Renewing JWT for user: {}", claims.username);
        // Poison outdated JWT
        PoisonableIdentifier::UserJWTId(claims.jwt_id_hex()).poison()?;

        let username = claims.username.clone();

        let claims = Claims::new(claims.username);

        match ServerApiSession::from_claims(claims) {
            Ok(ass) => Ok((jar.add(ass.build_cookie()), Json(ass.into()))),
            Err(e) => {
                log::error!(
                    "Error generating new session from_claims for username ({username}): {e:?}"
                );
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    pub async fn session_post(
        mut conn: DbConnHolder,
        jar: CookieJar,
        Json(payload): Json<PostSession>,
    ) -> Result<(CookieJar, Json<ApiSession>), StatusCode> {
        log::trace!("Generating new JWT");

        let session = match payload {
            PostSession::User(user) => {
                let db_hashed_password = users::get_user(
                    &mut conn.0,
                    users::Identifier::Username(user.username.as_str()),
                )
                .map_err(|e| match e {
                    db::Error::NotFound(error) => {
                        log::warn!(
                            "Tried to login to user: {username:?} but it doesn't exist: {error:?}",
                            username = &user.username
                        );
                        StatusCode::UNAUTHORIZED
                    }
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                })?
                .hashed_password;

                if !user.raw_password.password_matches_raw(&db_hashed_password) {
                    log::warn!("Passwords didn't match for payload: {user:?}");
                    return Err(StatusCode::UNAUTHORIZED);
                }

                if PoisonableIdentifier::Username(user.username.clone().into()).is_poisoned()? {
                    log::error!(
                        "Username should not be in DB when its poisoned, username: {:?}",
                        user.username
                    );
                    Err(StatusCode::INTERNAL_SERVER_ERROR)?
                }

                let claims = Claims::new(user.username.into());

                ServerApiSession::from_claims(claims)
            }
            PostSession::Sensor(sensor) => {
                log::info!("Generating sensor session for sensor: {sensor:?}");

                let signature_bytes = hex::decode(&sensor.signature_of_message)
                    .map_err(|e| {
                        log::warn!(
                            "Invalid signature received: {}, error: {e:?}",
                            sensor.signature_of_message
                        );
                        StatusCode::BAD_REQUEST
                    })?
                    .as_slice()
                    .try_into()
                    .map_err(|e| {
                        log::warn!("Invalid length of signature received: {e:?}");
                        StatusCode::BAD_REQUEST
                    })?;

                let signed_message = hex::decode(sensor.random_message_encoded).map_err(|e| {
                    log::warn!("User tried to get sensor session with invalid random_message_encoded: {e:?}");
                    StatusCode::BAD_REQUEST
                })?;

                AuthorizedSensor::from_signature_and_message(
                    &mut conn.0,
                    &sensor.device_id,
                    signature_bytes,
                    signed_message.as_slice(),
                )?;

                let claims = SensorClaims::new(sensor.device_id);
                log::warn!("SensorClaims: {claims:?}");
                ServerApiSession::from_sensor_claims(claims)
            }
        };

        match session {
            Ok(ass) => {
                log::info!("Session generated: {ass:?}");
                Ok((jar.add(ass.build_cookie()), Json(ass.into())))
            }
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
    use common::endpoints_io::session::UserLogin;

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

        Session::session_put(CookieJar::new(), claims)
            .await
            .expect("Should not fail");
    }

    #[tokio::test]
    async fn test_session_get() {
        let mut conn_nref = establish_connection(true).unwrap();
        let conn = &mut conn_nref;
        let (user, pass) = create_test_user(conn);

        let json = UserLogin {
            username: user.username.into(),
            raw_password: pass,
        };

        let json = PostSession::User(json);

        let conn = DbConnHolder(conn_nref);
        let _res = Session::session_post(conn, CookieJar::new(), Json(json))
            .await
            .expect("Should not fail");
    }
}
