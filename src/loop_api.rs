use std::io;
use std::cmp::max;

use time::SteadyTime;
use mio::{Token, Evented, EventSet, PollOpt, EventLoop, Timeout, TimerError};

use handler::{Handler, Timeo, EventMachine};


pub trait LoopApi {
    fn register(&mut self, io: &Evented, token: Token,
        interest: EventSet, opt: PollOpt) -> io::Result<()>;
    fn timeout(&mut self, token: Token, at: SteadyTime)
        -> Result<Timeout, TimerError>;
    fn clear_timeout(&mut self, token: Timeout) -> bool;
}

impl<'a, C, M> LoopApi for EventLoop<Handler<C, M>>
    where M: EventMachine<C>
{
    fn register(&mut self, io: &Evented, token: Token,
        interest: EventSet, opt: PollOpt) -> io::Result<()>
    {
        self.register(io, token, interest, opt)
    }

    fn timeout(&mut self, token: Token, at: SteadyTime)
        -> Result<Timeout, TimerError>
    {
        // is it too slow?
        let left = at - SteadyTime::now();
        self.timeout_ms(
            Timeo::Fsm(token),
            max(left.num_milliseconds(), 0) as u64,
        )
    }
    fn clear_timeout(&mut self, token: Timeout) -> bool
    {
        self.clear_timeout(token)
    }
}
