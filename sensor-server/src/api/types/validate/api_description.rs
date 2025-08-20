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
            && val.len() > Self::MAX_LEN
            && val.chars().all(|c| c.is_alphanumeric() || c == ' ')
        {
            Ok(())
        } else {
            Err(Error::Custom("Invalid description".into()))
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
