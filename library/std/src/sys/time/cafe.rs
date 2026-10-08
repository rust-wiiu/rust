use crate::sys::pal::abi;
use crate::sys::pal::ticks::ticks_to_duration;
use crate::time::Duration;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Instant(Duration);

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct SystemTime(Duration);

pub const UNIX_EPOCH: SystemTime = SystemTime(Duration::from_secs(0));

/// Seconds from 1970-01-01 (UNIX epoch) to 2000-01-01 (Cafe OS epoch).
const CAFE_EPOCH_OFFSET: Duration = Duration::from_secs(946_684_800);

impl Instant {
    pub fn now() -> Instant {
        let ticks = unsafe { abi::OSGetSystemTime() };
        Instant(ticks_to_duration(u64::try_from(ticks).unwrap_or(0)))
    }

    pub fn checked_sub_instant(&self, other: &Instant) -> Option<Duration> {
        self.0.checked_sub(other.0)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<Instant> {
        Some(Instant(self.0.checked_add(*other)?))
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<Instant> {
        Some(Instant(self.0.checked_sub(*other)?))
    }
}

impl SystemTime {
    pub const MAX: SystemTime = SystemTime(Duration::MAX);

    pub const MIN: SystemTime = SystemTime(Duration::ZERO);

    // Cafe OS returns local time, not UTC
    pub fn now() -> SystemTime {
        let ticks = unsafe { abi::OSGetTime() };
        SystemTime(CAFE_EPOCH_OFFSET + ticks_to_duration(u64::try_from(ticks).unwrap_or(0)))
    }

    pub fn sub_time(&self, other: &SystemTime) -> Result<Duration, Duration> {
        self.0.checked_sub(other.0).ok_or_else(|| other.0 - self.0)
    }

    pub fn checked_add_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime(self.0.checked_add(*other)?))
    }

    pub fn checked_sub_duration(&self, other: &Duration) -> Option<SystemTime> {
        Some(SystemTime(self.0.checked_sub(*other)?))
    }
}
