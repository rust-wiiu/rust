use crate::sync::atomic::Atomic;
use crate::sync::atomic::Ordering::Relaxed;
use crate::sys::pal::abi;
use crate::time::Duration;

pub type Futex = Atomic<Primitive>;
pub type Primitive = u32;

pub type SmallFutex = Atomic<SmallPrimitive>;
pub type SmallPrimitive = u32;

// Timer ticks per second: bus clock (248.625 MHz) / 4.
const TICKS_PER_SEC: u128 = 62_156_250;

fn to_ticks(dur: Duration) -> i64 {
    (dur.as_nanos() * TICKS_PER_SEC / 1_000_000_000).try_into().unwrap_or(i64::MAX)
}

// FIXME(cafe): polls with short sleeps; replace with real blocking waits.
// Sleeping rather than OSYieldThread lets lower-priority threads on the same core run.
pub fn futex_wait(futex: &Atomic<u32>, expected: u32, timeout: Option<Duration>) -> bool {
    let start = unsafe { abi::OSGetSystemTime() };
    let limit = timeout.map(to_ticks);
    loop {
        if futex.load(Relaxed) != expected {
            return true;
        }
        if let Some(limit) = limit {
            if unsafe { abi::OSGetSystemTime() }.saturating_sub(start) >= limit {
                return false;
            }
        }
        unsafe { abi::OSSleepTicks(to_ticks(Duration::from_micros(100))) };
    }
}

#[inline]
pub fn futex_wake(_futex: &Atomic<u32>) -> bool {
    false
}

#[inline]
pub fn futex_wake_all(_futex: &Atomic<u32>) {}
