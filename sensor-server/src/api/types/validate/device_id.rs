use serde::{Deserialize, Serialize};
use serde_valid::Validate;
use ts_rs::TS;

#[derive(Debug)]
pub enum Error {
    InvalidLength(i32),
    NotHex,
    NotLowercase,
}

#[derive(Deserialize, Serialize, Debug, Clone, TS, Eq, PartialEq, Hash, Validate)]
#[ts(export, export_to = "./api/types/")]
pub struct DeviceId(#[validate(custom(DeviceId::valid))] String);

impl DeviceId {
    pub const ID_LENGTH: usize = 40;

    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let id: String = (0..Self::ID_LENGTH)
            .map(|_| format!("{:x}", rng.random_range(0..16)))
            .collect();
        Self(id)
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    pub fn valid(val: &String) -> Result<(), serde_valid::validation::Error> {
        Self::test(val)
            .map_err(|e| serde_valid::validation::Error::Custom(format!("Invalid ApiId: {e:?}")))
    }

    /// Tests wether the id &str can be turned into ApiId
    pub fn test(id: &str) -> Result<(), Error> {
        if id.len() != Self::ID_LENGTH {
            return Err(Error::InvalidLength(
                (id.len() as i32) - (Self::ID_LENGTH as i32),
            ));
        }

        for c in id.chars() {
            if !c.is_ascii_hexdigit() {
                return Err(Error::NotHex);
            }
            if !((c.is_alphabetic() && c.is_lowercase()) || c.is_numeric()) {
                return Err(Error::NotLowercase);
            }
        }

        Ok(())
    }

    pub fn from_string(id: &str) -> Result<Self, Error> {
        Self::test(id)?;
        Ok(Self(id.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_api_id_creation() {
        let api_id = DeviceId::random();
        assert_eq!(api_id.0.len(), DeviceId::ID_LENGTH);
        assert!(api_id.0.chars().all(|c| c.is_ascii_hexdigit()
            && ((c.is_alphabetic() && c.is_lowercase()) || c.is_numeric())));
    }
}
