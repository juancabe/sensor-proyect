use axum::routing::MethodRouter;
use axum_serde_valid::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

use crate::{
    RoutePath,
    api::{
        Endpoint,
        endpoints::session::ApiSession,
        route::Route,
        types::{
            ApiTimestamp,
            validate::{
                api_email::ApiEmail, api_raw_password::ApiRawPassword, api_username::ApiUsername,
            },
        },
    },
    auth::claims::Claims,
    db::{
        DbConn, DbConnHolder,
        users::{Identifier, Update, get_user, insert_user, update_user},
    },
    model::NewUser,
    state::PoisonableIdentifier,
};

#[derive(TS, Debug, Serialize, Deserialize, Validate, Clone)]
#[ts(export, export_to = "./api/endpoints/user/")]
pub struct ApiUser {
    #[validate]
    pub username: ApiUsername,
    #[validate]
    pub email: ApiEmail,
    pub created_at: ApiTimestamp,
    pub updated_at: ApiTimestamp,
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/user/")]
pub struct GetUser {}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate, Clone)]
#[ts(export, export_to = "./api/endpoints/user/")]
pub enum PutUser {
    Username(#[validate] ApiUsername),
    RawPassword(#[validate] ApiRawPassword),
    Email(#[validate] ApiEmail),
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate, Clone)]
#[ts(export, export_to = "./api/endpoints/user/")]
// WARN: Dont accept this in any endpoint
pub struct PutUserResponse {
    pub updated: ApiUser,
    pub new_session: ApiSession,
}

/// Register User
#[derive(TS, Debug, serde::Serialize, serde::Deserialize, Validate)]
#[ts(export, export_to = "./api/endpoints/user/")]
pub struct PostUser {
    #[validate]
    pub username: ApiUsername,
    #[validate]
    pub raw_password: ApiRawPassword,
    #[validate]
    pub email: ApiEmail,
}

#[derive(TS, Debug, serde::Serialize, serde::Deserialize, PartialEq, Validate)]
#[ts(export, export_to = "./api/endpoints/user/")]
// WARN: Do not acccept this in any endpoints
pub enum NotUniqueUser {
    Username(String),
    Email(String),
}

pub struct User {
    resources: Vec<Route>,
}

impl User {
    pub const API_PATH: &str = "/user";
    pub fn new() -> User {
        let mr = MethodRouter::new()
            .get(Self::user_get)
            .post(Self::user_post) // Register
            .put(Self::user_put); // Update

        Self {
            resources: vec![Route::new(
                RoutePath::from_string(Self::API_PATH.to_string())
                    .expect("The route should be correct"),
                mr,
            )],
        }
    }

    async fn user_get(claims: Claims, mut conn: DbConnHolder) -> Result<Json<ApiUser>, StatusCode> {
        let conn = &mut conn.0;
        let user = get_user(conn, Identifier::Username(&claims.username))?;

        let username = user.username.into();
        let email = user.email.into();
        let created_at = user.created_at.and_utc().timestamp() as usize;
        let updated_at = user.updated_at.and_utc().timestamp() as usize;

        let au = ApiUser {
            username,
            email,
            created_at,
            updated_at,
        };

        log::trace!("User got: {au:?}");

        Ok(Json(au))
    }

    /// Should not be called with Identifier::Id
    fn user_field_exists_for_some_user(
        conn: &mut DbConn,
        identifier: Identifier,
        user_id: Option<i32>, // and that user_id is not this
    ) -> Result<bool, StatusCode> {
        if let Identifier::Id(_) = identifier {
            log::error!("This function was not meant to be called with an ID as identifier");
            assert!(false);
        }

        match get_user(conn, identifier) {
            Ok(user) => {
                log::trace!("On user_field_exists_for_some_user, colliding user was: {user:?}");
                Ok(user_id.is_none_or(|uid| uid != user.id))
            }
            Err(e) => match e {
                crate::db::Error::NotFound(_) => Ok(false),
                _ => Err(e.into()),
            },
        }
    }

    /// Update a user attribute
    async fn user_put(
        mut conn: DbConnHolder,
        claims: Claims,
        Json(payload): Json<PutUser>,
    ) -> Result<Json<PutUserResponse>, StatusCode> {
        let conn = &mut conn.0;

        let user = get_user(conn, Identifier::Username(&claims.username))?;

        log::trace!("User requesting update is: {user:?}");

        // Check for repeated fields
        let identifier = match &payload {
            PutUser::Username(un) => Some(Identifier::Username(un.as_str())),
            PutUser::RawPassword(_) => None,
            PutUser::Email(em) => Some(Identifier::Email(em.as_str())),
        };

        if let Some(identifier) = identifier {
            match Self::user_field_exists_for_some_user(conn, identifier.clone(), Some(user.id)) {
                Ok(exists) => {
                    if exists {
                        log::trace!("user_put failed, same value: {identifier:?}");
                        Err(StatusCode::CONFLICT)?
                    }
                }
                Err(e) => Err(e)?,
            }
        }

        // Check for poisoned
        let identifier = match &payload {
            PutUser::Username(un) => Some(PoisonableIdentifier::Username(un.clone().into())),
            PutUser::RawPassword(_) => None,
            PutUser::Email(em) => Some(PoisonableIdentifier::Email(em.clone().into())),
        };

        if let Some(identifier) = identifier {
            match identifier.is_poisoned() {
                Ok(poisoned) => {
                    if poisoned {
                        log::warn!("user_put failed, poisoned value: {identifier:?}");
                        Err(StatusCode::CONFLICT)?
                    }
                }
                Err(e) => Err(e)?,
            }
        }

        let user = update_user(
            conn,
            Identifier::Username(&claims.username),
            payload.clone() as Update,
        )?;

        log::trace!("User updated to: {user:?}");

        // Poison last identifier
        let identifier = match &payload {
            PutUser::Username(un) => Some(PoisonableIdentifier::Username(un.clone().into())),
            PutUser::RawPassword(_) => None,
            PutUser::Email(em) => Some(PoisonableIdentifier::Email(em.clone().into())),
        };

        if let Some(identifier) = identifier {
            match identifier.poison() {
                Ok(()) => {
                    log::trace!("Identifier: {identifier:?} poisoned");
                }
                Err(e) => Err(e)?,
            }
        }

        // Poison used JWT Id
        let id = PoisonableIdentifier::JWTId(claims.jwt_id_hex());
        id.poison()?;
        log::trace!("Identifier: {id:?} poisoned");

        // Return updated
        let crate::model::User {
            id: _,
            username,
            hashed_password: _,
            email,
            created_at,
            updated_at,
            updated_auth_at: _,
        } = user;

        let created_at = created_at.and_utc().timestamp() as ApiTimestamp;
        let updated_at = updated_at.and_utc().timestamp() as ApiTimestamp;

        let username: ApiUsername = username.into();
        let email = email.into();

        let new_session =
            ApiSession::from_claims(Claims::new(username.clone().into())).map_err(|e| {
                log::error!("Could not construct new_session from claims: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        let updated = ApiUser {
            username,
            email,
            created_at,
            updated_at,
        };

        Ok(Json(PutUserResponse {
            updated,
            new_session,
        }))
    }

    async fn user_post(
        mut conn: DbConnHolder,
        Json(payload): Json<PostUser>,
    ) -> (StatusCode, Json<Option<NotUniqueUser>>) {
        let conn = &mut conn.0;

        log::trace!("user_post body received: {payload:?}");

        let PostUser {
            username,
            raw_password,
            email,
        } = payload;

        // Check if attempting to register with poisoned username
        match PoisonableIdentifier::Username(username.clone().into()).is_poisoned() {
            Ok(is) => {
                if is {
                    log::warn!("User tried to register poisoned username: {username:?}");
                    return (
                        StatusCode::CONFLICT,
                        Json(Some(NotUniqueUser::Username(username.into()))),
                    );
                }
            }
            Err(e) => {
                log::error!("Error checking is_poisoned: {e:?}");
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
            }
        }

        // Check if attempting to register with poisoned email
        match PoisonableIdentifier::Email(email.clone().into()).is_poisoned() {
            Ok(is) => {
                if is {
                    log::warn!("User tried to register poisoned email: {email:?}");
                    return (
                        StatusCode::CONFLICT,
                        Json(Some(NotUniqueUser::Email(email.into()))),
                    );
                }
            }
            Err(e) => {
                log::error!("Error checking is_poisoned: {e:?}");
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(None));
            }
        }

        match Self::user_field_exists_for_some_user(
            conn,
            Identifier::Username(username.as_str()),
            None,
        ) {
            Ok(exists) => {
                if exists {
                    log::trace!("username was repeated: {username:?}");
                    return (
                        StatusCode::CONFLICT,
                        Json(Some(NotUniqueUser::Username(username.into()))),
                    );
                }
            }
            Err(e) => return (e.into(), Json(None)),
        }

        match Self::user_field_exists_for_some_user(conn, Identifier::Email(email.as_str()), None) {
            Ok(exists) => {
                if exists {
                    log::trace!("email was repeated: {email:?}");
                    return (
                        StatusCode::CONFLICT,
                        Json(Some(NotUniqueUser::Email(email.into()))),
                    );
                }
            }
            Err(e) => return (e.into(), Json(None)),
        }

        let hashed_password = match raw_password.hash() {
            Ok(p) => p,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
        };

        let new_user = NewUser {
            username: username.into(),
            hashed_password: hashed_password.into(),
            email: email.into(),
        };

        log::trace!("NewUser: {new_user:?}");

        match insert_user(conn, new_user) {
            Ok(_) => (StatusCode::OK, Json(None)),
            Err(e) => (e.into(), Json(None)),
        }
    }
}

impl Endpoint for User {
    fn routes(&self) -> &[Route] {
        return &self.resources;
    }
    fn path(&self) -> &str {
        Self::API_PATH
    }
}

#[cfg(test)]
mod test {
    use axum_serde_valid::Json;
    use hyper::StatusCode;

    use crate::{
        api::{
            endpoints::user::{PostUser, PutUser, User},
            types::validate::{
                api_email::ApiEmail, api_raw_password::ApiRawPassword, api_username::ApiUsername,
            },
        },
        auth::claims::Claims,
        db::{DbConnHolder, establish_connection, tests::create_test_user},
        model,
    };

    #[tokio::test]
    async fn test_user_get() {
        let mut conn = establish_connection(true).unwrap();

        let (user, _) = create_test_user(&mut conn);

        let mut claims = Claims::new(user.username.clone());

        let res = User::user_get(claims.clone(), DbConnHolder(conn))
            .await
            .expect("Should not fail");

        assert_eq!(res.username, user.username.into());
        assert_eq!(res.email, user.email.into());

        let mut conn = establish_connection(true).unwrap();
        let _user = create_test_user(&mut conn);
        claims.username = "anotheruser".into();
        let res = User::user_get(claims, DbConnHolder(conn))
            .await
            .err()
            .expect("Should fail");

        assert_eq!(res, StatusCode::NOT_FOUND)
    }

    #[tokio::test]
    async fn test_user_post() {
        let conn = establish_connection(true).unwrap();

        let username = ApiUsername::random();
        let raw_password = ApiRawPassword::random();
        let _hashed_password = raw_password.hash().expect("Should hash");
        let email = ApiEmail::random();

        let json = PostUser {
            username,
            raw_password,
            email,
        };

        let (code, _string) = User::user_post(DbConnHolder(conn), Json(json)).await;
        assert_eq!(code, StatusCode::OK);

        let mut conn = establish_connection(true).unwrap();

        let (
            model::User {
                id: _,
                username,
                hashed_password: _,
                email,
                created_at: _,
                updated_at: _,
                updated_auth_at: _,
            },
            raw_password,
        ) = create_test_user(&mut conn);

        let username = username.into();
        let email = email.into();

        let json = PostUser {
            username,
            raw_password,
            email,
        };

        let (code, _string) = User::user_post(DbConnHolder(conn), Json(json)).await;
        assert_eq!(code, StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_user_put() {
        let mut conn = establish_connection(true).unwrap();
        let (
            model::User {
                id: _,
                username,
                hashed_password: _,
                email: _,
                created_at: _,
                updated_at: _,
                updated_auth_at: _,
            },
            _raw_password,
        ) = create_test_user(&mut conn);

        let new_username = ApiUsername::random();

        let json = PutUser::Username(new_username.clone());

        assert_ne!(&username, new_username.as_str());

        let claims = Claims::new(username);

        let res = User::user_put(DbConnHolder(conn), claims, Json(json))
            .await
            .expect("Should not fail");
        assert_eq!(res.updated.username, new_username);
    }

    #[tokio::test]
    async fn test_user_put_fail() {
        let mut conn = establish_connection(true).unwrap();

        let (user1, _) = create_test_user(&mut conn);
        let (user2, _) = create_test_user(&mut conn);

        let new_username_user2 = user1.username;

        let json = PutUser::Username(new_username_user2.clone().into());

        let claims = Claims::new(user2.username);

        let res = User::user_put(DbConnHolder(conn), claims, Json(json))
            .await
            .err()
            .expect("Should fail");

        assert_eq!(res, StatusCode::CONFLICT);
    }
}
