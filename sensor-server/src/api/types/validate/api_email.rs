use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, PartialEq)]
#[ts(export, export_to = "./api/types/")]
pub struct ApiEmail(#[validate(pattern = r"^[^\s@]+@[^\s@]+\.[^\s@]+$")] String);

impl ApiEmail {
    pub const MAX_LEN: usize = 320;
    pub const MIN_LEN: usize = 3;

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn random() -> Self {
        Self(
            rand::rng()
                .sample_iter(&Alphanumeric)
                .take(rand::random_range(Self::MIN_LEN..=Self::MAX_LEN - 20))
                .map(char::from)
                .collect::<String>()
                + "@email.com",
        )
    }
}

impl From<String> for ApiEmail {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ApiEmail> for String {
    fn from(value: ApiEmail) -> Self {
        value.0
    }
}
