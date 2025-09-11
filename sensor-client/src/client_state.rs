use common::auth::keys::Keys;
use serde::{Deserialize, Serialize};

use crate::wifi_connector::WifiClientConfig;

#[derive(Serialize, Deserialize, Debug)]
pub struct Persistence {
    pub wifi: WifiClientConfig,
    pub auth_keys: Keys,
}

#[derive(Debug)]
pub enum State {
    Uninitialized,
    UnsetPersistence,
    InconsistentPersistence, // At least one of the keys was set and other unset
    PersistenceGot,
}

pub struct ClientState(State);

impl ClientState {
    pub fn new() -> Self {
        Self(State::Uninitialized)
    }

    pub fn get_state(&self) -> &State {
        &self.0
    }

    pub fn apply_persistence(
        &mut self,
        wifi: Option<WifiClientConfig>,
        auth_keys: Option<Keys>,
    ) -> Option<Persistence> {
        let mut ret = None;
        *self = match (wifi, auth_keys) {
            (Some(wifi), Some(auth_keys)) => {
                ret = Some(Persistence { wifi, auth_keys });
                Self(State::PersistenceGot)
            }
            (None, None) => Self(State::UnsetPersistence),
            _ => Self(State::InconsistentPersistence),
        };
        ret
    }
}
