use esp_idf_svc::nvs::{EspNvs, NvsDefault};

pub const NAMESPACE: &'static str = "default";

/// Keys enum
/// - Represents the available keys on the persistence storage
/// - Must be initialized with a str
pub enum Keys {
    WifiConfigSerialized,
    AuthKeysSerialized,
}

impl Keys {
    pub const fn key_string(&self) -> &'static str {
        match self {
            Keys::WifiConfigSerialized => "wifi_config",
            Keys::AuthKeysSerialized => "auth_keys",
        }
    }
    // str lenghth limit for JWTs is 2047 characters, str buffer size is 2048: need 1 byte for null terminator
    pub const fn max_value_length(&self) -> usize {
        match self {
            Keys::WifiConfigSerialized => 1 << 10, // 2^10
            Keys::AuthKeysSerialized => 1 << 10,   // 2^10
        }
    }

    pub const fn min_recv_buffer_size(&self) -> usize {
        match self {
            Keys::WifiConfigSerialized => self.max_value_length() + 1,
            Keys::AuthKeysSerialized => self.max_value_length() + 1,
        }
    }
}

pub type Excess = usize;
pub type Lack = usize;

#[derive(Debug)]
pub enum Error {
    CreationError(esp_idf_sys::EspError),
    RemoveError(esp_idf_sys::EspError),
    SizeLimitExceeded(Excess),
    LackingBuffer(Lack),
    SetError(esp_idf_sys::EspError),
    GetError(esp_idf_sys::EspError),
}

pub struct Persistence {
    nvs: EspNvs<NvsDefault>,
}

impl Persistence {
    pub fn new(nvs_p: esp_idf_svc::nvs::EspDefaultNvsPartition) -> Result<Self, Error> {
        let nvs = EspNvs::new(nvs_p, NAMESPACE, true).map_err(|e| Error::CreationError(e))?;
        Ok(Persistence { nvs })
    }

    pub fn set(&mut self, key: Keys, value: &str) -> Result<(), Error> {
        match key {
            // UserApiId
            Keys::WifiConfigSerialized => {
                if value.len() > key.max_value_length() {
                    return Err(Error::SizeLimitExceeded(
                        value.len() - key.max_value_length(),
                    ));
                }
                self.nvs
                    .set_str(key.key_string(), value)
                    .map_err(|e| Error::SetError(e))?;
            }
            // SensorApiId
            Keys::AuthKeysSerialized => {
                if value.len() > key.max_value_length() {
                    return Err(Error::SizeLimitExceeded(
                        value.len() - key.max_value_length(),
                    ));
                }

                self.nvs
                    .set_str(key.key_string(), value)
                    .map_err(|e| Error::SetError(e))?;
            } // Keys::AuthKeysSerialized => {
              //     self.nvs
              //         .remove(key.key_string())
              //         .map_err(Error::CreationError)?;
              // }
        }

        Ok(())
    }

    // Returns false on NOT FOUND
    pub fn remove(&mut self, key: Keys) -> Result<bool, Error> {
        self.nvs
            .remove(key.key_string())
            .map_err(Error::RemoveError)
    }

    pub fn get(&self, key: Keys, buf: &mut [u8]) -> Result<bool, Error> {
        if buf.len() < (key.min_recv_buffer_size()) {
            return Err(Error::LackingBuffer(key.min_recv_buffer_size() - buf.len()));
        }

        match key {
            Keys::WifiConfigSerialized => {
                let res = self
                    .nvs
                    .get_str(key.key_string(), buf)
                    .map_err(|e| Error::GetError(e))?;
                Ok(res.is_some())
            }
            Keys::AuthKeysSerialized => {
                let res = self
                    .nvs
                    .get_str(key.key_string(), buf)
                    .map_err(|e| Error::GetError(e))?;
                Ok(res.is_some())
            }
        }
    }
}
