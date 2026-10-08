use crate::io;
use crate::num::NonZero;
use crate::sys::pal::abi;
use crate::sys::pal::ticks::duration_to_ticks;
use crate::time::Duration;

pub fn sleep(dur: Duration) {
    unsafe { abi::OSSleepTicks(duration_to_ticks(dur)) }
}

pub fn yield_now() {
    unsafe { abi::OSYieldThread() }
}

pub fn available_parallelism() -> io::Result<NonZero<usize>> {
    let cores = unsafe { abi::OSGetCoreCount() } as usize;
    NonZero::new(cores).ok_or(io::Error::UNKNOWN_THREAD_COUNT)
}
