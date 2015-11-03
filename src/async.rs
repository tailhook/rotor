use std::cmp::min;

use time::SteadyTime;


#[derive(PartialEq, Eq, Debug)]
#[must_use]
pub enum Async<M, V> {
    Continue(M, V),
    Stop,
    Timeout(M, SteadyTime),
}

impl<M, V> Async<M, V> {
    pub fn and_then<T, R, F: FnOnce(M) -> Async<T, R>>(self, f: F)
        -> Async<T, R>
    {
        use self::Async::*;
        match self {
            Continue(m, _) => f(m),
            Stop => Stop,
            Timeout(m, t1) => match f(m) {
                Continue(m, v) => Continue(m, v),
                Stop => Stop,
                Timeout(m, t2) => Timeout(m, min(t1, t2)),
            },
        }
    }
    pub fn map<T, F: FnOnce(M) -> T>(self, f: F) -> Async<T, V> {
        use self::Async::*;
        match self {
            Continue(m, v) => Continue(f(m), v),
            Stop => Stop,
            Timeout(m, t) => Timeout(f(m), t),
        }
    }
    pub fn map_result<R, F: FnOnce(V) -> R>(self, f: F) -> Async<M, R> {
        use self::Async::*;
        match self {
            Continue(m, v) => Continue(m, f(v)),
            Stop => Stop,
            Timeout(m, t) => Timeout(m, t),
        }
    }
    pub fn done<R, F: FnOnce(M) -> R>(self, f: F) -> Option<R> {
        use self::Async::*;
        match self {
            Continue(m, _) => Some(f(m)),
            Stop => None,
            Timeout(m, _) => Some(f(m)),
        }
    }
}

#[macro_export]
macro_rules! async_try {
    ($e:expr) => {
        match $e {
            $crate::async::Async::Continue(m, v)
            => $crate::async::Async::Continue(m, v),
            $crate::async::Async::Timeout(m, t)
            => $crate::async::Async::Timeout(m, t),
            $crate::async::Async::Stop
            => return $crate::async::Async::Stop,
        }
    }
}
