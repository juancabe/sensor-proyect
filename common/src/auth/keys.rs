use ed25519_dalek::{Signature, SigningKey, VerifyingKey, ed25519::signature::SignerMut};
// use esp_idf_sys::esp_fill_random;

pub struct Keys {
    sk: SigningKey,
    vk: VerifyingKey,
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
