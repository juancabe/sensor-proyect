#[derive(Debug)]
pub enum ZStr20Error {
    SizeNotMatch,
    FromHexError(hex::FromHexError),
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ZStr20 {
    pub bytes: [u8; 21],
}

impl ZStr20 {
    pub const LENGTH: usize = 20;
    pub const SIZE: usize = Self::LENGTH + 1;

    pub fn new(bytes: &[u8; Self::LENGTH]) -> Self {
        let mut zstr = ZStr20 {
            bytes: [0; Self::SIZE],
        };
        zstr.bytes[..Self::LENGTH].copy_from_slice(bytes);
        zstr.bytes[Self::LENGTH] = 0; // Null-terminate
        zstr
    }

    pub fn from_unsized_slice(slice: &[u8]) -> Result<Self, ZStr20Error> {
        Ok(ZStr20::new(
            slice.try_into().map_err(|_| ZStr20Error::SizeNotMatch)?,
        ))
    }

    pub fn as_hex_string(&self) -> String {
        hex::encode(&self.bytes[..Self::LENGTH])
    }

    pub fn from_hex_string(hex: &str) -> Result<Self, ZStr20Error> {
        if hex.len() != Self::LENGTH * 2 {
            log::error!(
                "Expected len: {}, got len: {} for str: {hex}",
                Self::LENGTH,
                hex.len(),
            );
            Err(ZStr20Error::SizeNotMatch)?
        }

        let bytes = hex::decode(hex).map_err(|e| ZStr20Error::FromHexError(e))?;
        ZStr20::from_unsized_slice(bytes.as_slice())
    }

    pub fn inner_info_slice(&self) -> &[u8] {
        &self.bytes[..Self::SIZE]
    }

    pub fn multiple_to_arbitrary_bytes(zstrs: &[Self], desired_vec_size: usize) -> Vec<u8> {
        let mut vec = vec![];
        for zstr in zstrs {
            if vec.len() >= desired_vec_size {
                assert!(vec.len() == desired_vec_size);
                return vec;
            }

            let to_extend_bytes = usize::min(Self::LENGTH, desired_vec_size - vec.len());
            vec.extend(&zstr.bytes[..to_extend_bytes]);
        }
        // If not enough bytes from zstrs, fill with 0s
        let to_fill_bytes = desired_vec_size - vec.len();
        let os = vec![0u8; to_fill_bytes];
        vec.extend(os);
        vec
    }

    pub fn arbitrary_bytes_to_multiple(mut arbitrary: &[u8]) -> Vec<Self> {
        let mut vec = vec![];

        while arbitrary.len() >= Self::LENGTH {
            let zstr = &arbitrary[..Self::LENGTH];
            let zstr = ZStr20::from_unsized_slice(zstr)
                .expect("Should be correct as arbitrary.len() >= Self::LENGTH");
            vec.push(zstr);

            // Reslice arbitrary
            arbitrary = &arbitrary[Self::LENGTH..];
        }

        // Arbitrary can contain 0 - 20 bytes left
        if arbitrary.len() > 0 {
            let mut last = vec![0u8; 20];
            last[..arbitrary.len()].copy_from_slice(arbitrary);
            let last = ZStr20::from_unsized_slice(&last).expect("len should be equal to 20");
            vec.push(last);
        }

        vec
    }
}
