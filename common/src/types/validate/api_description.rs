use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_valid::{Validate, validation::Error};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, PartialEq)]
#[ts(export, export_to = "./api/types/")]
pub struct ApiDescription(#[validate(custom(ApiDescription::valid))] String);

impl ApiDescription {
    pub const MIN_LEN: usize = 5;
    pub const MAX_LEN: usize = 50;

    fn valid(val: &String) -> Result<(), serde_valid::validation::Error> {
        if val.len() < Self::MIN_LEN
            || val.len() > Self::MAX_LEN
            || val.chars().any(|c| !(c.is_alphanumeric() || c == ' '))
            || val.trim() != val
        {
            Err(Error::Custom("Invalid description".into()))
        } else {
            Ok(())
        }
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
}

impl From<String> for ApiDescription {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ApiDescription> for String {
    fn from(value: ApiDescription) -> Self {
        value.0
    }
}

#[cfg(test)]
mod test {
    use serde_valid::Validate;

    use crate::types::validate::api_description::ApiDescription;

    #[test]
    fn test_api_description_success() {
        ApiDescription::from("This is a valid description".to_string())
            .validate()
            .expect("Should be fine");
        ApiDescription::from("This is a valid description 1231231 too".to_string())
            .validate()
            .expect("Should be fine");
        ApiDescription::from("123This is a valid description".to_string())
            .validate()
            .expect("Should be fine");
        ApiDescription::from("a".repeat(ApiDescription::MIN_LEN).to_string())
            .validate()
            .expect("Should be fine");
        ApiDescription::from("a".repeat(ApiDescription::MAX_LEN).to_string())
            .validate()
            .expect("Should be fine");
    }

    #[test]
    fn test_api_description_fail() {
        ApiDescription::from("This is a inva[lid description".to_string())
            .validate()
            .expect_err("Should error");
        ApiDescription::from("This is a valid? description 1231231 too".to_string())
            .validate()
            .expect_err("Should error");
        ApiDescription::from("123This is a \n not valid description".to_string())
            .validate()
            .expect_err("Should error");
        ApiDescription::from("a".repeat(ApiDescription::MIN_LEN - 1).to_string())
            .validate()
            .expect_err("Should error");
        ApiDescription::from("a".repeat(ApiDescription::MAX_LEN + 1).to_string())
            .validate()
            .expect_err("Should error");
    }
}
