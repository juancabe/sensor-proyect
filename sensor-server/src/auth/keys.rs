use std::sync::LazyLock;

use jsonwebtoken::{DecodingKey, EncodingKey};
use rand::{TryRngCore, rngs::OsRng};

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub static KEYS: LazyLock<Keys> = LazyLock::new(|| {
    // Generate per-runtime Keys
    const LEN: usize = 512;

    let mut buff = vec![0u8; LEN];
    OsRng
        .try_fill_bytes(&mut buff)
        .expect("OsRng should be able to generate random");
    Keys::new(&buff)
});
