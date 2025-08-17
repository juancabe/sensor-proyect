use chrono::TimeDelta;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Claims {
    pub username: String,
    pub iat: usize,
    pub exp: usize,
}

impl Claims {
    pub fn new(username: String) -> Claims {
        let now = chrono::Utc::now();
        let expires_in = TimeDelta::days(1);
        let tomorrow = now
            .checked_add_signed(expires_in)
            .expect("Should not be out of range");

        Claims {
            username: username,
            iat: now.timestamp() as usize,
            exp: tomorrow.timestamp() as usize,
        }
    }
}
