use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug)]
pub enum Error {
    InvalidLength(i32),
    NotHex,
    NotLowercase,
}

#[derive(Deserialize, Serialize, Debug, Clone, TS, Eq, PartialEq, Hash)]
#[ts(export)]
pub struct ApiId {
    // ID_LENGTH lowercase hex characters for a unique identifier
    id: String,
}

impl ApiId {
    pub const ID_LENGTH: usize = 40;

    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::rng();
        let id: String = (0..Self::ID_LENGTH)
            .map(|_| format!("{:x}", rng.random_range(0..16)))
            .collect();
        Self { id }
    }

    pub fn to_string(&self) -> String {
        self.id.clone()
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
        Ok(Self { id: id.to_string() })
    }

    pub fn as_str(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_api_id_creation() {
        let api_id = ApiId::random();
        assert_eq!(api_id.id.len(), ApiId::ID_LENGTH);
        assert!(api_id.id.chars().all(|c| c.is_ascii_hexdigit()
            && ((c.is_alphabetic() && c.is_lowercase()) || c.is_numeric())));
    }
}
