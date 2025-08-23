use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use serde_valid::{Validate, validation::Error};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, PartialEq)]
#[ts(export, export_to = "./api/types/")]
pub struct ApiEntityName(#[validate(custom(ApiEntityName::valid))] String);

impl ApiEntityName {
    pub const MAX_LEN: usize = 15;
    pub const MIN_LEN: usize = 3;

    fn valid(val: &String) -> Result<(), serde_valid::validation::Error> {
        if val.len() > Self::MAX_LEN
            || val.len() < Self::MIN_LEN
            || val.chars().any(|c| !(c.is_alphanumeric() || c == ' '))
            || val.trim() != val
        {
            Err(Error::Custom("Invalid username".into()))
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

impl From<String> for ApiEntityName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ApiEntityName> for String {
    fn from(value: ApiEntityName) -> Self {
        value.0
    }
}

#[cfg(test)]
mod test {
    use serde_valid::Validate;

    use crate::api::types::validate::api_entity_name::ApiEntityName;

    #[test]
    fn test_api_description_success() {
        ApiEntityName::from("ValidName".to_string())
            .validate()
            .expect("Should be fine");
        ApiEntityName::from("Valid Name Too".to_string())
            .validate()
            .expect("Should be fine");
        ApiEntityName::from("Valid 123 TOO".to_string())
            .validate()
            .expect("Should be fine");
        ApiEntityName::from("a".repeat(ApiEntityName::MIN_LEN).to_string())
            .validate()
            .expect("Should be fine");
        ApiEntityName::from("a".repeat(ApiEntityName::MAX_LEN).to_string())
            .validate()
            .expect("Should be fine");
    }

    #[test]
    fn test_api_description_fail() {
        ApiEntityName::from("inva[lid name".to_string())
            .validate()
            .expect_err("Should error");
        ApiEntityName::from(" is a invalid".to_string())
            .validate()
            .expect_err("Should error");
        ApiEntityName::from("123 is a \n".to_string())
            .validate()
            .expect_err("Should error");
        ApiEntityName::from("a".repeat(ApiEntityName::MIN_LEN - 1).to_string())
            .validate()
            .expect_err("Should error");
        ApiEntityName::from("a".repeat(ApiEntityName::MAX_LEN + 1).to_string())
            .validate()
            .expect_err("Should error");
    }
}
