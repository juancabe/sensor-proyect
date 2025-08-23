use std::sync::{LazyLock, Mutex};

use chrono::TimeDelta;
use hyper::StatusCode;
use jsonwebtoken::{Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{api::types::validate::device_id::DeviceId, auth::keys::PROCESS_KEYS, state};

static ID_COUNTER: LazyLock<Mutex<u128>> = LazyLock::new(|| Mutex::new(u128::default()));
pub fn get_new_id() -> u128 {
    let mut lock = ID_COUNTER.lock().expect("Mutex should unlock");
    *lock += 1;
    *lock
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
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
        encode(&Header::default(), &self, &PROCESS_KEYS.encoding)
    }

    /// common logic to decode + poison-check
    pub fn from_jwt(jwt: &str) -> Result<Self, StatusCode> {
        let token_data =
            decode::<SensorClaims>(jwt, &PROCESS_KEYS.decoding, &Validation::default()).map_err(
                |e| {
                    log::warn!("Unable to decode JWT: {e:?}");
                    StatusCode::UNAUTHORIZED
                },
            )?;

        // Check state poisoned status
        if state::poisonable_identifier::PoisonableIdentifier::SensorJWTId(token_data.claims.jwt_id_hex()).is_poisoned()? {
            log::warn!("Tried to access with poisoned JWT: {jwt}, token_data: {token_data:?}");
            return Err(StatusCode::UNAUTHORIZED);
        }
        if state::poisonable_identifier::PoisonableIdentifier::DeviceID(token_data.claims.device_id.to_string())
            .is_poisoned()?
        {
            log::warn!(
                "Tried to access with poisoned username, JWT: {jwt}, token_data: {token_data:?}"
            );
            return Err(StatusCode::UNAUTHORIZED);
        }

        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        api::types::validate::device_id::DeviceId, auth::sensor_claims::SensorClaims,
        state::poisonable_identifier::PoisonableIdentifier,
    };

    #[test]
    fn test_claims() {
        let claims = SensorClaims::new(DeviceId::random());
        let jwt = claims.encode_jwt().unwrap();
        let new_claims = SensorClaims::from_jwt(jwt.as_str()).expect("Should not be poisoned");
        assert_eq!(claims, new_claims);
    }

    #[test]
    fn test_claims_fail() {
        let claims = SensorClaims::new(DeviceId::random());

        PoisonableIdentifier::SensorJWTId(claims.jwt_id_hex())
            .poison()
            .expect("Should not fail on poisoning");

        let jwt = claims.encode_jwt().unwrap();
        SensorClaims::from_jwt(jwt.as_str()).expect_err("Should be poisoned");
    }
}
