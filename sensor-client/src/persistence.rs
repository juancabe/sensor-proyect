use esp_idf_svc::nvs::{EspNvs, NvsDefault};

pub const NAMESPACE: &'static str = "default";

pub enum Keys<'a> {
    JWT(Option<&'a str>),
}

impl<'a> Keys<'a> {
    pub const fn key_string(&self) -> &'static str {
        match self {
            Keys::JWT(_) => "jwt",
        }
    }
    // str lenghth limit for JWTs is 2047 characters, str buffer size is 2048: need 1 byte for null terminator
    pub const fn max_value_length(&self) -> usize {
        match self {
            Keys::JWT(_) => 2047,
        }
    }

    pub const fn min_buffer_size(&self) -> usize {
        match self {
            Keys::JWT(_) => self.max_value_length() + 1, // + 1 for null terminator
        }
    }
}

pub type Excess = usize;
pub type Lack = usize;

#[derive(Debug)]
pub enum Error {
    CreationError(esp_idf_sys::EspError),
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

    pub fn set(&mut self, key: Keys<'_>) -> Result<(), Error> {
        match key {
            Keys::JWT(Some(value)) => {
                if value.len() > key.max_value_length() {
                    return Err(Error::SizeLimitExceeded(
                        value.len() - key.max_value_length(),
                    ));
                }

                self.nvs
                    .set_str(key.key_string(), value)
                    .map_err(|e| Error::SetError(e))?;
            }
            Keys::JWT(None) => {
                self.nvs
                    .remove(key.key_string())
                    .map_err(Error::CreationError)?;
            }
        }

        Ok(())
    }

    pub fn get(&self, key: Keys<'_>, buf: &mut [u8]) -> Result<bool, Error> {
        if buf.len() < (key.min_buffer_size()) {
            return Err(Error::LackingBuffer(key.min_buffer_size() - buf.len()));
        }

        match key {
            Keys::JWT(_) => {
                let res = self
                    .nvs
                    .get_str(key.key_string(), buf)
                    .map_err(|e| Error::GetError(e))?;
                Ok(res.is_some())
            }
        }
    }
}
