pub use core::time::Duration;

use core::{
    cmp::Ordering,
    ops::{Add, AddAssign, Sub, SubAssign},
};

/// A `no_std`-friendly stand-in for `std::time::Instant`.
///
/// - If the target has 64-bit atomics, `Instant::now()` is monotonic-ish (a counter).
/// - Otherwise, `Instant::now()` is always the same instant (so elapsed = 0).
///
/// This is meant as a compile-time fallback, not a real clock.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct Instant {
    ticks: u64,
}

// A "tick" is an abstract unit; we treat 1 tick == 1ns for Duration conversions.
const TICK_NANOS: u64 = 1;

impl Instant {
    /// Returns an `Instant` corresponding to "now".
    #[inline]
    pub fn now() -> Self {
        Self {
            ticks: monotonic_ticks(),
        }
    }

    /// Returns the amount of time elapsed since this instant was created.
    #[inline]
    pub fn elapsed(&self) -> Duration {
        Self::now().saturating_duration_since(*self)
    }

    /// Returns the duration between two instants, or panics if `earlier` is later than `self`.
    #[inline]
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        match self.ticks.cmp(&earlier.ticks) {
            Ordering::Less => {
                panic!("stub::time::Instant::duration_since: earlier > self")
            }
            _ => ticks_to_duration(self.ticks - earlier.ticks),
        }
    }

    /// Returns the duration between two instants, or `None` if `earlier` is later than `self`.
    #[inline]
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        self.ticks.checked_sub(earlier.ticks).map(ticks_to_duration)
    }

    /// Returns the duration between two instants, saturating at zero if `earlier` is later than `self`.
    #[inline]
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        match self.ticks.checked_sub(earlier.ticks) {
            Some(dt) => ticks_to_duration(dt),
            None => Duration::from_secs(0),
        }
    }

    /// Returns `Some(t)` where `t` is the time `self + duration` if it does not overflow.
    #[inline]
    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        let dt = duration_to_ticks(duration)?;
        self.ticks.checked_add(dt).map(|t| Instant { ticks: t })
    }

    /// Returns `Some(t)` where `t` is the time `self - duration` if it does not underflow.
    #[inline]
    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        let dt = duration_to_ticks(duration)?;
        self.ticks.checked_sub(dt).map(|t| Instant { ticks: t })
    }
}

impl Ord for Instant {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.ticks.cmp(&other.ticks)
    }
}

impl PartialOrd for Instant {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;
    #[inline]
    fn add(self, rhs: Duration) -> Instant {
        self.checked_add(rhs)
            .expect("stub::time::Instant + Duration overflow")
    }
}

impl AddAssign<Duration> for Instant {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;
    #[inline]
    fn sub(self, rhs: Duration) -> Instant {
        self.checked_sub(rhs)
            .expect("stub::time::Instant - Duration underflow")
    }
}

impl SubAssign<Duration> for Instant {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

impl Sub<Instant> for Instant {
    type Output = Duration;
    #[inline]
    fn sub(self, rhs: Instant) -> Duration {
        self.duration_since(rhs)
    }
}

/// A conservative stand-in for `std::time::SystemTime`.
///
/// In this stub:
/// - `SystemTime::now()` returns `UNIX_EPOCH` (so it compiles deterministically).
/// - `duration_since` / `elapsed` behave consistently based on that.
/// If you prefer to make it error instead, tell me and I’ll flip it.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct SystemTime {
    // represent as duration since UNIX_EPOCH
    since_epoch: Duration,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct SystemTimeError {
    _priv: (),
}

impl SystemTimeError {
    #[inline]
    pub fn duration(&self) -> Duration {
        Duration::from_secs(0)
    }
}

pub const UNIX_EPOCH: SystemTime = SystemTime {
    since_epoch: Duration::from_secs(0),
};

impl SystemTime {
    #[inline]
    pub fn now() -> SystemTime {
        // Deterministic fallback: "time does not advance".
        UNIX_EPOCH
    }

    #[inline]
    pub fn duration_since(
        &self,
        earlier: SystemTime,
    ) -> Result<Duration, SystemTimeError> {
        match self.since_epoch.checked_sub(earlier.since_epoch) {
            Some(d) => Ok(d),
            None => Err(SystemTimeError { _priv: () }),
        }
    }

    #[inline]
    pub fn elapsed(&self) -> Result<Duration, SystemTimeError> {
        SystemTime::now().duration_since(*self)
    }

    #[inline]
    pub fn checked_add(&self, duration: Duration) -> Option<SystemTime> {
        self.since_epoch
            .checked_add(duration)
            .map(|d| SystemTime { since_epoch: d })
    }

    #[inline]
    pub fn checked_sub(&self, duration: Duration) -> Option<SystemTime> {
        self.since_epoch
            .checked_sub(duration)
            .map(|d| SystemTime { since_epoch: d })
    }
}

// ---- internals ----

#[inline]
fn ticks_to_duration(ticks: u64) -> Duration {
    // 1 tick = 1ns (arbitrary but convenient)
    let nanos = ticks.saturating_mul(TICK_NANOS);
    Duration::from_nanos(nanos)
}

#[inline]
fn duration_to_ticks(d: Duration) -> Option<u64> {
    let secs = d.as_secs();
    let sub = d.subsec_nanos() as u64;

    // total_nanos = secs * 1_000_000_000 + sub
    let a = secs.checked_mul(1_000_000_000)?;
    let total = a.checked_add(sub)?;
    Some(total / TICK_NANOS)
}

#[inline]
fn monotonic_ticks() -> u64 {
    // If we have atomics, make `now()` monotonic-ish via a global counter.
    // Otherwise: deterministic "frozen time".
    #[cfg(target_has_atomic = "64")]
    {
        use core::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        // Start at 1 to avoid "all zeros" if you care.
        COUNTER.fetch_add(1, Ordering::Relaxed) + 1
    }

    #[cfg(not(target_has_atomic = "64"))]
    {
        0
    }
}
