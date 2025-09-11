use core::fmt;

use ed25519_dalek::{Signature, SigningKey, VerifyingKey, ed25519::signature::SignerMut};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, IgnoredAny, MapAccess, Visitor},
    ser::SerializeStruct,
};
// use esp_idf_sys::esp_fill_random;

#[derive(Debug, Clone)]
pub struct Keys {
    sk: SigningKey,
    vk: VerifyingKey,
}

impl Serialize for Keys {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Keys", 1)?;
        state.serialize_field("sk", &hex::encode(self.sk.as_bytes()))?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Keys {
    fn deserialize<D>(deserializer: D) -> Result<Keys, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Sk,
            Vk,
        }

        struct KeysVisitor;

        impl<'de> Visitor<'de> for KeysVisitor {
            type Value = Keys;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("struct Keys with `sk` (and optionally `vk`)")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Keys, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut sk_hex: Option<String> = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Sk => {
                            if sk_hex.is_some() {
                                return Err(de::Error::duplicate_field("sk"));
                            }
                            sk_hex = Some(map.next_value()?);
                        }
                        Field::Vk => {
                            // Ignore or parse if desired:
                            let _ = map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let sk_hex = sk_hex.ok_or_else(|| de::Error::missing_field("sk"))?;
                let bytes = hex::decode(sk_hex)
                    .map_err(|e| de::Error::custom(format!("Invalid HEX: {e}")))?;
                let bytes = <[u8; 32]>::try_from(bytes.as_slice())
                    .map_err(|_| de::Error::custom("Invalid slice size for SigningKey"))?;
                let sk = SigningKey::from_bytes(&bytes);
                Ok(Keys::from_sk(sk))
            }
        }

        const FIELDS: &[&str] = &["sk", "vk"];
        deserializer.deserialize_struct("Keys", FIELDS, KeysVisitor)
    }
}

impl Keys {
    const PUBLIC_KEY_LENGTH: usize = 32;

    pub fn new(random_seed: &[u8; 32]) -> Self {
        // let mut seed = [0u8; 32];
        //
        // unsafe {
        //     esp_fill_random(seed.as_mut_ptr() as *mut core::ffi::c_void, seed.len());
        // }

        let sk = SigningKey::from_bytes(&random_seed);

        Self::from_sk(sk)
    }

    pub fn get_vk(&self) -> [u8; Self::PUBLIC_KEY_LENGTH] {
        self.vk.to_bytes()
    }

    pub fn from_sk(sk: SigningKey) -> Self {
        let vk = sk.verifying_key();
        Self { sk, vk }
    }

    pub fn sign(&mut self, msg: &[u8]) -> Signature {
        self.sk.sign(msg)
    }
}
