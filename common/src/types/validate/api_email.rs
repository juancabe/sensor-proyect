use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, PartialEq)]
#[ts(export, export_to = "./api/types/")]
// The hypothetically valid mail a@b doesn't match, but idc
pub struct ApiEmail(
    #[validate(pattern = r"^[^\s@]+@[^\s@]+\.[^\s@]+$")]
    #[validate(max_length = 320)]
    #[validate(min_length = 4)]
    String,
);

impl ApiEmail {
    pub const MAX_LEN: usize = 320;
    pub const MIN_LEN: usize = 4;

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

#[cfg(test)]
mod test {
    use serde_valid::Validate;

    use crate::types::validate::api_email::ApiEmail;

    const SUFFIX: &str = "@gmail.com";

    #[test]
    fn test_api_email_success() {
        ApiEmail::from("juan@gmail.com".to_string())
            .validate()
            .expect("Should be fine");
        ApiEmail::from("juan@a.c".to_string())
            .validate()
            .expect("Should be fine");
        ApiEmail::from("juan.cb@email.es".to_string())
            .validate()
            .expect("Should be fine");
        ApiEmail::from("a@b.c".to_string())
            .validate()
            .expect("Should be fine");
        ApiEmail::from("a".repeat(ApiEmail::MAX_LEN - SUFFIX.len()).to_string() + SUFFIX)
            .validate()
            .expect("Should be fine");
    }

    #[test]
    fn test_api_email_fail() {
        ApiEmail::from("invalid.mail".to_string())
            .validate()
            .expect_err("Should error");
        ApiEmail::from("invalid.mail@".to_string())
            .validate()
            .expect_err("Should error");
        ApiEmail::from("invalid".to_string())
            .validate()
            .expect_err("Should error");
        ApiEmail::from("a@".to_string())
            .validate()
            .expect_err("Should error");
        ApiEmail::from("a".repeat(ApiEmail::MAX_LEN + 2 - SUFFIX.len()).to_string() + SUFFIX)
            .validate()
            .expect_err("Should error");
    }
}
