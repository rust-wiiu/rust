use cafe_sys::nlibnss;

use crate::sync::Once;

pub fn fill_bytes(bytes: &mut [u8]) {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let result = unsafe { nlibnss::init() };
        assert!(result == nlibnss::Error::OK, "failed to initialize nlibnss");
    });
    for chunk in bytes.chunks_mut(i32::MAX as usize) {
        let result = unsafe { nlibnss::fill_random(chunk.as_mut_ptr().cast(), chunk.len() as i32) };
        assert!(result == nlibnss::Error::OK, "failed to generate random data");
    }
}
