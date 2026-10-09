use crate::sync::Once;
use crate::sys::pal::abi;

pub fn fill_bytes(bytes: &mut [u8]) {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let result = unsafe { abi::NSSInit() };
        assert_eq!(result, 0, "failed to initialize nlibnss");
    });
    for chunk in bytes.chunks_mut(i32::MAX as usize) {
        let result = unsafe { abi::NSSGetRandom(chunk.as_mut_ptr(), chunk.len() as i32) };
        assert_eq!(result, 0, "failed to generate random data");
    }
}
