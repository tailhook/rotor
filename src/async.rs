use std::cmp::min;

use time::SteadyTime;


#[derive(PartialEq, Eq, Debug)]
#[must_use]
pub enum Async<M:Sized, V:Sized, S:Sized> {
    Send(M, V),
    Yield(M, S),
    Return(M, V, S), // Send + Yield
    Ignore(M),
    Stop,
}

impl<M:Sized, V:Sized, S:Sized> Async<M, V, S> {
    /*
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
    */
    pub fn map<T, F: FnOnce(M) -> T>(self, f: F) -> Async<T, V, S> {
        use self::Async::*;
        match self {
            Send(m, v) => Send(f(m), v),
            Yield(m, s) => Yield(f(m), s),
            Return(m, v, s) => Return(f(m), v, s),
            Ignore(m) => Ignore(f(m)),
            Stop => Stop,
        }
    }
    pub fn map_result<R, F: FnOnce(V) -> R>(self, f: F) -> Async<M, R, S> {
        use self::Async::*;
        match self {
            Send(m, v) => Send(m, f(v)),
            Yield(m, s) => Yield(m, s),
            Return(m, v, s) => Return(m, f(v), s),
            Ignore(m) => Ignore(m),
            Stop => Stop,
        }
    }
    /*
    pub fn done<R, F: FnOnce(M) -> R>(self, f: F) -> Option<R> {
        use self::Async::*;
        match self {
            Continue(m, _) => Some(f(m)),
            Stop => None,
            Timeout(m, _) => Some(f(m)),
        }
    }
    */
}

/*
impl<M> Async<M, Option<M>> {
    pub fn wrap<T, F: FnMut(M) -> T>(self, mut f: F) -> Async<T, Option<T>> {
        use self::Async::*;
        match self {
            Continue(m, v) => Continue(f(m), v.map(f)),
            Stop => Stop,
            Timeout(m, t) => Timeout(f(m), t),
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
*/
