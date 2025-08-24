use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

use crate::{
    endpoints_io::session::ApiSession,
    types::{
        ApiTimestamp,
        validate::{
            api_email::ApiEmail, api_raw_password::ApiRawPassword, api_username::ApiUsername,
        },
    },
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
