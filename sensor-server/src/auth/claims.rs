use std::sync::{LazyLock, Mutex};

use chrono::TimeDelta;
use jsonwebtoken::{Header, encode};
use serde::{Deserialize, Serialize};

use crate::auth::keys::KEYS;

static ID_COUNTER: LazyLock<Mutex<u128>> = LazyLock::new(|| Mutex::new(u128::default()));
pub fn get_new_id() -> u128 {
    let mut lock = ID_COUNTER.lock().expect("Mutex should unlock");
    *lock += 1;
    *lock
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub jwt_id: u128,
    pub username: String,
    pub iat: usize,
    pub exp: usize,
}

impl Claims {
    pub fn new(username: String) -> Claims {
        let now = chrono::Utc::now();
        let expires_in = TimeDelta::days(1);
        let tomorrow = now
            .checked_add_signed(expires_in)
            .expect("Should not be out of range");

        Claims {
            jwt_id: get_new_id(),
            username: username,
            iat: now.timestamp() as usize,
            exp: tomorrow.timestamp() as usize,
        }
    }

    pub fn jwt_id_hex(&self) -> String {
        format!("{:x}", self.jwt_id)
    }

    pub fn encode_jwt(&self) -> Result<String, jsonwebtoken::errors::Error> {
        encode(&Header::default(), &self, &KEYS.encoding)
    }
}
