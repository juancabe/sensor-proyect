use std::{
    collections::HashMap,
    fmt::Display,
    sync::{LazyLock, Mutex},
};

use hyper::StatusCode;

use crate::api::types::ApiTimestamp;

type PoisoningMap = LazyLock<Mutex<HashMap<String, ApiTimestamp>>>;

static POISONED_JWTS: PoisoningMap = LazyLock::new(|| Mutex::new(HashMap::new()));
static POISONED_USERNAMES: PoisoningMap = LazyLock::new(|| Mutex::new(HashMap::new()));
static POISONED_EMAILS: PoisoningMap = LazyLock::new(|| Mutex::new(HashMap::new()));

type ExternalError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum Error {
    LockError(ExternalError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::LockError(error) => write!(f, "LockError: {}", error.to_string()),
        }
    }
}

impl std::error::Error for Error {}

impl From<Error> for StatusCode {
    fn from(value: Error) -> Self {
        match value {
            Error::LockError(e) => {
                log::error!(
                    "Turning PoisonableIdentifier Error::LockError into INTERNAL_SERVER_ERROR, e: {e:?}"
                );
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PoisonableIdentifier {
    // Hex that identifies a JWT
    JWTId(String),
    Username(String),
    Email(String),
}

impl PoisonableIdentifier {
    pub const POISON_TIME: ApiTimestamp = 10 * 60; // 10 minutes

    fn now() -> ApiTimestamp {
        chrono::Utc::now().timestamp() as ApiTimestamp
    }

    fn poison_key(map: &PoisoningMap, key: String) -> Result<(), Error> {
        log::trace!("Poisoning key: {key}");
        map.lock()
            .map_err(|e| Error::LockError(e.to_string().into()))?
            .insert(key, Self::now() + Self::POISON_TIME);
        Ok(())
    }

    fn key_is_poisoned(map: &PoisoningMap, key: &str) -> Result<bool, Error> {
        let res: Option<bool> = map
            .lock()
            .map_err(|e| Error::LockError(e.to_string().into()))?
            .get(key)
            .and_then(|p_until| Some(*p_until > Self::now()));
        Ok(res.is_some_and(|time_remained| time_remained))
    }

    fn as_key(&self) -> &String {
        match self {
            PoisonableIdentifier::JWTId(k) => k,
            PoisonableIdentifier::Username(k) => k,
            PoisonableIdentifier::Email(k) => k,
        }
    }

    fn into_key(&self) -> String {
        match self {
            PoisonableIdentifier::JWTId(k) => k.clone(),
            PoisonableIdentifier::Username(k) => k.clone(),
            PoisonableIdentifier::Email(k) => k.clone(),
        }
    }

    fn associated_map(&self) -> &PoisoningMap {
        match self {
            PoisonableIdentifier::JWTId(_) => &POISONED_JWTS,
            PoisonableIdentifier::Username(_) => &POISONED_USERNAMES,
            PoisonableIdentifier::Email(_) => &POISONED_EMAILS,
        }
    }

    pub fn poison(&self) -> Result<(), Error> {
        Self::poison_key(self.associated_map(), self.into_key())
    }

    pub fn is_poisoned(&self) -> Result<bool, Error> {
        Self::key_is_poisoned(self.associated_map(), &self.as_key())
    }
}
