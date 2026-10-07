use crate::sync::atomic::Ordering::Relaxed;
use crate::sync::atomic::{Atomic, AtomicUsize};
use crate::sys::pal::abi;

pub type Key = usize;

// Cafe OS has 16 thread-specific slots per thread. Slot 0 is skipped
// because 0 is the "not yet created" sentinel of `LazyKey`.
const SLOT_COUNT: usize = 16;
static NEXT_KEY: Atomic<usize> = AtomicUsize::new(1);

// FIXME(cafe): destructors are not run when a thread exits.
pub fn create(_dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    let key = NEXT_KEY.fetch_add(1, Relaxed);
    if key >= SLOT_COUNT {
        rtabort!("out of TLS keys");
    }
    key
}

#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    unsafe { abi::OSSetThreadSpecific(key as i32, value.cast()) }
}

#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    unsafe { abi::OSGetThreadSpecific(key as i32).cast() }
}

#[inline]
pub unsafe fn destroy(_key: Key) {}
