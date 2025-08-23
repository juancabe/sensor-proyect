use std::sync::LazyLock;

use jsonwebtoken::{DecodingKey, EncodingKey};
use rand::{TryRngCore, rngs::OsRng};

/// ProcessKeys struct, all ProcessKeys consumers should not expect them to persist if process restarts
/// ## Lifetime
/// Generated on process start, destroyed on process end
pub struct ProcessKeys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl ProcessKeys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub static PROCESS_KEYS: LazyLock<ProcessKeys> = LazyLock::new(|| {
    // Generate per-process Keys
    const LEN: usize = 512;

    let mut buff = vec![0u8; LEN];
    OsRng
        .try_fill_bytes(&mut buff)
        .expect("OsRng should be able to generate random");
    ProcessKeys::new(&buff)
});
