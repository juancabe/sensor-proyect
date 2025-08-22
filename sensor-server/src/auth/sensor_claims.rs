use std::sync::{LazyLock, Mutex};

use chrono::TimeDelta;
use jsonwebtoken::{Header, encode};
use serde::{Deserialize, Serialize};

use crate::{api::types::validate::device_id::DeviceId, auth::keys::KEYS};

static ID_COUNTER: LazyLock<Mutex<u128>> = LazyLock::new(|| Mutex::new(u128::default()));
pub fn get_new_id() -> u128 {
    let mut lock = ID_COUNTER.lock().expect("Mutex should unlock");
    *lock += 1;
    *lock
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SensorClaims {
    pub jwt_id: u128,
    pub device_id: String,
    pub iat: usize,
    pub exp: usize,
}

impl SensorClaims {
    pub fn new(device_id: DeviceId) -> Self {
        let now = chrono::Utc::now();
        let expires_in = TimeDelta::days(1);
        let tomorrow = now
            .checked_add_signed(expires_in)
            .expect("Should not be out of range");

        let claims = SensorClaims {
            jwt_id: get_new_id(),
            device_id: device_id.to_string(),
            iat: now.timestamp() as usize,
            exp: tomorrow.timestamp() as usize,
        };

        log::trace!("SensorClaims generated: {claims:?}");

        claims
    }

    pub fn jwt_id_hex(&self) -> String {
        format!("{:x}", self.jwt_id)
    }

    pub fn encode_jwt(&self) -> Result<String, jsonwebtoken::errors::Error> {
        encode(&Header::default(), &self, &KEYS.encoding)
    }
}
