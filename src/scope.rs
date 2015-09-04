use std::io;

use mio::{Timeout, TimerError, Evented, EventSet, PollOpt};


pub trait Scope<Machine, Timer> {
    fn async_add_machine(&mut self, m: Machine) -> Result<(), Machine>;
    fn add_timeout_ms(&mut self, delay: u64, t: Timer)
        -> Result<Timeout, TimerError>;
    fn clear_timeout(&mut self, timeout: Timeout) -> bool;
    fn register<E: ?Sized>(&mut self, io: &E, interest: EventSet, opt: PollOpt)
        -> Result<(), io::Error>
        where E: Evented;
}
