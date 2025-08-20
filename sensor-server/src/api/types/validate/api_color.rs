use serde::{Deserialize, Serialize};
use serde_valid::{Validate, validation::Error};
use ts_rs::TS;

use crate::model::COLOR_HEX_STRS;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, TS, PartialEq)]
#[ts(export, export_to = "./api/types/")]
pub struct ApiColor(#[validate(custom(ApiColor::valid))] String);

impl ApiColor {
    pub const LEN: usize = 7;

    fn valid(val: &String) -> Result<(), serde_valid::validation::Error> {
        if COLOR_HEX_STRS.contains(&val.as_str()) {
            Ok(())
        } else {
            return Err(Error::Custom("Invalid color chars".into()));
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn random() -> Self {
        COLOR_HEX_STRS[rand::random_range(0..COLOR_HEX_STRS.len())]
            .to_string()
            .into()
    }
}

impl From<String> for ApiColor {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<ApiColor> for String {
    fn from(value: ApiColor) -> Self {
        value.0
    }
}
