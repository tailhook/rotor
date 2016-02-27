use std::io;

use mio::{Token, EventLoop};

use handler::{Handler, Timeo};
use {Machine};
use {Evented, EventSet, PollOpt, Timeout, TimerError};


#[doc(hidden)]
pub trait LoopApi {
    fn register(&mut self, io: &Evented, token: Token,
        interest: EventSet, opt: PollOpt) -> io::Result<()>;
    fn reregister(&mut self, io: &Evented, token: Token,
        interest: EventSet, opt: PollOpt) -> io::Result<()>;
    fn deregister(&mut self, io: &Evented) -> io::Result<()>;
    fn timeout_ms(&mut self, token: Token, delay: u64)
        -> Result<Timeout, TimerError>;
    fn clear_timeout(&mut self, token: Timeout) -> bool;
    fn shutdown(&mut self);
}

impl<'a, M: Machine> LoopApi for EventLoop<Handler<M>>
{
    fn register(&mut self, io: &Evented, token: Token,
        interest: EventSet, opt: PollOpt) -> io::Result<()>
    {
        self.register(io, token, interest, opt)
    }

    fn reregister(&mut self, io: &Evented, token: Token,
        interest: EventSet, opt: PollOpt) -> io::Result<()>
    {
        self.reregister(io, token, interest, opt)
    }

    fn deregister(&mut self, io: &Evented) -> io::Result<()>
    {
        self.deregister(io)
    }

    fn timeout_ms(&mut self, token: Token, delay: u64)
        -> Result<Timeout, TimerError>
    {
        self.timeout_ms( Timeo::Fsm(token), delay)
    }
    fn clear_timeout(&mut self, token: Timeout) -> bool
    {
        self.clear_timeout(token)
    }
    fn shutdown(&mut self) {
        self.shutdown()
    }
}
