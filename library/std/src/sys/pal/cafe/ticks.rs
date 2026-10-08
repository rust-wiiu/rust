#![expect(dead_code)]
use super::abi;
use crate::time::Duration;

/// Frequency of the time base read by `OSGetTime` and `OSGetSystemTime`.
pub fn ticks_per_second() -> u64 {
    unsafe { u64::from((*abi::OSGetSystemInfo()).bus_clock_speed) / 4 }
}

pub fn ticks_to_duration(ticks: u64) -> Duration {
    let tps = ticks_per_second();
    let nanos = (ticks % tps) * 1_000_000_000 / tps;
    Duration::new(ticks / tps, nanos as u32)
}

/// Rounds up, so that waits never end early, and saturates at `i64::MAX`.
pub fn duration_to_ticks(dur: Duration) -> i64 {
    let ticks = (dur.as_nanos() * u128::from(ticks_per_second())).div_ceil(1_000_000_000);
    ticks.try_into().unwrap_or(i64::MAX)
}
