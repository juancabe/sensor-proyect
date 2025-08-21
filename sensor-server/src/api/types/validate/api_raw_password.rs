use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use password_hash::{SaltString, rand_core::OsRng};
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_valid::{Validate, validation::Error};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, PartialEq)]
#[ts(export, export_to = "./api/types/")]
pub struct ApiRawPassword(#[validate(custom(ApiRawPassword::valid))] String);

impl ApiRawPassword {
    pub const MIN_LEN: usize = 5;
    pub const MAX_LEN: usize = 64;

    fn valid(val: &String) -> Result<(), serde_valid::validation::Error> {
        if val.len() < Self::MIN_LEN || val.len() > Self::MAX_LEN {
            Err(Error::Custom(format!(
                "Password must have between {min}-{max} characters",
                min = Self::MIN_LEN,
                max = Self::MAX_LEN
            )))?
        }

        if val.chars().any(char::is_control) {
            Err(Error::Custom(format!(
                "Password cannot include control characters"
            )))?
        }

        if val != val.trim() {
            Err(Error::Custom(format!(
                "Password cannot start or end with whitespaces"
            )))?
        }

        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn random() -> Self {
        Self(
            rand::rng()
                .sample_iter(&Alphanumeric)
                .take(rand::random_range(Self::MIN_LEN..=Self::MAX_LEN))
                .map(char::from)
                .collect(),
        )
    }

    pub fn hash(&self) -> Result<String, password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(self.0.as_bytes(), &salt)
            .inspect_err(|e| log::error!("Hash error: {e:?}"))?;
        Ok(hash.to_string())
    }

    pub fn password_matches_raw(&self, stored_hash: &str) -> bool {
        if let Ok(parsed_hash) = PasswordHash::new(stored_hash) {
            Argon2::default()
                .verify_password(self.as_str().as_bytes(), &parsed_hash)
                .is_ok()
        } else {
            false
        }
    }
}

impl From<String> for ApiRawPassword {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ApiRawPassword> for String {
    fn from(value: ApiRawPassword) -> Self {
        value.0
    }
}
