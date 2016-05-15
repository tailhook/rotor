
use std::ops::Add;
use std::time::{Duration, Instant, SystemTime};

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


fn millis(dur: Duration) -> u64 {
    dur.as_secs()*1000 + (dur.subsec_nanos()/1000000) as u64
}

impl Add<Duration> for Time {
    type Output = Time;
    fn add(self, rhs: Duration) -> Time {
        Time(self.0 + millis(rhs))
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

pub fn make_time(base: Instant, now: Instant) -> Time {
    Time(millis(now.duration_since(base))
         // Time starts with 1 not with zero
         + 1)
}

pub fn mio_timeout_ms(now: Time, event: Time) -> u64 {
    if event.0 > now.0 {
        // We need +1 because we truncate both old and new timeouts to
        // millisecond precision, while mio calculates at the nanosecond
        // precision (but doesn't expose it). So wake up time may be up
        // to a millisecond smaller then expected
        event.0 - now.0 + 1
    } else {
        0
    }
}

pub fn estimate_system_time(now: Time, value: Time) -> SystemTime {
    SystemTime::now() + Duration::from_millis(value.0 - now.0)
}


#[cfg(test)]
mod test {
    use super::Time;
    use std::time::Duration;


    #[test]
    fn test_add_duration() {
        let tm = Time::zero();
        assert_eq!(tm + Duration::new(10, 0), Time(10001));
        assert_eq!(tm + Duration::from_millis(150), Time(151));
        assert_eq!(tm + Duration::from_millis(12345), Time(12346));
        assert_eq!(tm + Duration::new(5, 0) + Duration::from_millis(20),
                   Time(5021));
    }

}
