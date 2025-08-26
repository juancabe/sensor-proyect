use ed25519_dalek::{PUBLIC_KEY_LENGTH, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_valid::{Validate, validation::Error};
use ts_rs::TS;

use crate::auth::keys::Keys;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, PartialEq)]
#[ts(export, export_to = "./api/types/")]
pub struct ApiPubKey(#[validate(custom(ApiPubKey::valid))] String);

impl ApiPubKey {
    pub const LEN: usize = PUBLIC_KEY_LENGTH * 2; // HEX encoded, so double

    fn valid(val: &String) -> Result<(), serde_valid::validation::Error> {
        if val.len() != Self::LEN {
            Err(Error::Custom(format!("Invalid Pub Key lenght",)))?
        }

        let decoded = match hex::decode(val) {
            Ok(data) => data,
            Err(_) => Err(Error::Custom("Invalid HEX".into()))?,
        };

        let decoded_slice: [u8; PUBLIC_KEY_LENGTH] = match decoded.try_into() {
            Ok(slice) => slice,
            Err(_) => {
                log::error!("[ApiPubKey] validate: unexpected double invalid lenght");
                Err(Error::Custom("Invalid lenght".into()))?
            }
        };

        match VerifyingKey::from_bytes(&decoded_slice) {
            Ok(_) => Ok(()),
            Err(e) => {
                log::warn!("Error forming VerifyingKey from {val}: {e:?}");
                Err(Error::Custom("Invalid public key supplied".into()))
            }
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn random(seed: &[u8; 32]) -> Self {
        Self(hex::encode(Keys::new(seed).get_vk()))
    }
}

impl From<String> for ApiPubKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ApiPubKey> for String {
    fn from(value: ApiPubKey) -> Self {
        value.0
    }
}
