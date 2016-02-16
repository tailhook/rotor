use time::SteadyTime;

use std::ops::Add;
use std::time::Duration;

/// The current time
///
/// This value is similar to (and directly derived from) the
/// `time::SteadyTime`.  But it has three important properties:
///
/// 1. It has a millisecond precision
/// 2. It's size is 8 bytes (SteadyTime is 16 bytes)
/// 3. It supports math with `std::time::Duration` (more future-proof)
///
/// The size of the value is important because we are going to have a lot of
/// timeouts and lots of timestamps stored inside the state machines.
///
/// Precision of millisecond is good enough, and even better for our use case
/// as it allows faster comparison and more frequent matches.
///
/// Warning: when adding a duration that is not a multiple of a millisecond
/// we truncate (i.e. floor) the duration value. We may change this in future.
/// Note that for timeouts this works well enough, as mio already bumps the
/// timeout to at least a millisecond ahead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Time(u64);

impl Add<Duration> for Time {
    type Output = Time;
    fn add(self, rhs: Duration) -> Time {
        Time(self.0 + rhs.as_secs() + (rhs.subsec_nanos()/1000000) as u64)
    }
}

impl Time {
    /// Zero time value, should be used only as a starting point for unit
    /// tests
    pub fn zero() -> Time {
        // In fact we don't care actual value, but the 1 allows us to
        // implement NonZero in the future
        Time(1)
    }
}

pub fn make_time(base: SteadyTime, now: SteadyTime) -> Time {
    Time((now - base).num_milliseconds() as u64)
}

pub fn diff_ms(now: Time, event: Time) -> u64 {
    if event.0 > now.0 {
        event.0 - now.0
    } else {
        0
    }
}
