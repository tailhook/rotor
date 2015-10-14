use std::fmt::Display;

use time::SteadyTime;


#[derive(PartialEq, Eq, Debug)]
pub enum Async<M, V> {
    Continue(M, V),
    Stop,
    Timeout(M, SteadyTime),
}

impl<M, V> Async<M, V> {
    pub fn map<T, F: FnOnce(M) -> T>(self, f: F) -> Async<T, V> {
        use self::Async::*;
        match self {
            Continue(m, v) => Continue(f(m), v),
            Stop => Stop,
            Timeout(m, t) => Timeout(f(m), t),
        }
    }
}
