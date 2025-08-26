use esp_idf_sys::esp_fill_random;

pub mod zstr20;

pub fn get_random_buf<const SIZE: usize>() -> [u8; SIZE] {
    let mut seed = [0u8; SIZE];

    unsafe {
        esp_fill_random(seed.as_mut_ptr() as *mut core::ffi::c_void, seed.len());
    }

    seed
}
