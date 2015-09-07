use std::io;

use mio::{Timeout, TimerError, Evented, EventSet, PollOpt};

use BaseMachine;


pub trait Scope<M:BaseMachine> {
    fn async_add_machine(&mut self, m: M) -> Result<(), M>;
    fn add_timeout_ms(&mut self, delay: u64, t: M::Timeout)
        -> Result<Timeout, TimerError>;
    fn clear_timeout(&mut self, timeout: Timeout) -> bool;
    fn register<E: ?Sized>(&mut self, io: &E, interest: EventSet, opt: PollOpt)
        -> Result<(), io::Error>
        where E: Evented;
}
