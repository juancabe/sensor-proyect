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
        &self.bytes[..Self::LENGTH]
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
            let mut last = vec![0u8; Self::LENGTH];
            last[..arbitrary.len()].copy_from_slice(arbitrary);
            let last = ZStr20::from_unsized_slice(&last).expect("len should be equal to 20");
            vec.push(last);
        }

        vec
    }
}

#[cfg(test)]
mod test {
    use crate::types::zstr20::ZStr20;
    #[test]
    fn test_create() {
        let slice = [1u8; 20];
        ZStr20::new(&slice);
        ZStr20::from_unsized_slice(&slice).expect("Should be valid ZStr20");
        let vec = ZStr20::arbitrary_bytes_to_multiple(&slice);
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn test_multiple_to_arbitrary() {
        let slices = [
            [1u8; ZStr20::LENGTH],
            [2u8; ZStr20::LENGTH],
            [3u8; ZStr20::LENGTH],
            [4u8; ZStr20::LENGTH],
            [5u8; ZStr20::LENGTH],
        ];
        let zstrs: Vec<ZStr20> = slices.iter().map(|slice| ZStr20::new(slice)).collect();
        let created_bytes =
            ZStr20::multiple_to_arbitrary_bytes(&zstrs, slices.len() * ZStr20::LENGTH);
        assert_eq!(created_bytes.len(), slices.len() * ZStr20::LENGTH);
        for (index, slice) in slices.iter().enumerate() {
            assert_eq!(
                slice,
                &created_bytes[index * ZStr20::LENGTH..(index + 1) * ZStr20::LENGTH]
            );
        }

        // Now with an extra ZStr20 filled with 0s
        let created_bytes =
            ZStr20::multiple_to_arbitrary_bytes(&zstrs, (slices.len() + 1) * (ZStr20::LENGTH));
        for (index, slice) in slices.iter().enumerate() {
            assert_eq!(
                slice,
                &created_bytes[index * ZStr20::LENGTH..(index + 1) * ZStr20::LENGTH]
            );
        }
        assert_eq!(
            [0u8; ZStr20::LENGTH],
            created_bytes[(slices.len()) * ZStr20::LENGTH..(slices.len() + 1) * ZStr20::LENGTH]
        );
    }

    #[test]
    fn test_arbitrary_to_multiple() {
        let arbitrary = &[8u8; 435];
        let res = ZStr20::arbitrary_bytes_to_multiple(arbitrary);
        assert_eq!(res.len(), arbitrary.len() / ZStr20::LENGTH + 1);
        for bytes in res[0..res.len() - 1].iter().map(|zstr| zstr.bytes) {
            assert_eq!([8u8; ZStr20::LENGTH], bytes[0..ZStr20::LENGTH]);
        }
        // Last ZStr20 may contain zeros for padding
        for byte in res.last().unwrap().bytes {
            assert!(byte == 8u8 || byte == 0u8)
        }
    }
}
