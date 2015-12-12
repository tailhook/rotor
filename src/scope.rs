use std::io;
use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};

use mio::{Token, Sender, Evented, EventSet, PollOpt, Timeout, TimerError};

use {Notify, Future, Port, LoopApi};

pub struct Scope<'a, C:Sized+'a>{
    token: Token,
    ctx: &'a mut C,
    channel: &'a mut Sender<Notify>,
    loop_api: &'a mut LoopApi,
}

impl<'a, C:Sized+'a> Scope<'a, C> {
    pub fn register(&mut self, io: &Evented, interest: EventSet, opt: PollOpt)
        -> io::Result<()>
    {
        self.loop_api.register(io, self.token, interest, opt)
    }

    pub fn timeout_ms(&mut self, delay: u64) -> Result<Timeout, TimerError>
    {
        self.loop_api.timeout_ms(self.token, delay)
    }
    pub fn clear_timeout(&mut self, token: Timeout) -> bool
    {
        self.loop_api.clear_timeout(token)
    }
}

fn pair<T:Sized>(token: Token, channel: &Sender<Notify>)
    -> (Port<T>, Future<T>)
{
    let arc = Arc::new(Mutex::new(None::<T>));
    let port = Port {
        token: token,
        contents: arc.clone(),
        channel: channel.clone(),
    };
    let future = Future {
        contents: arc,
    };
    return (port, future);
}


impl<'a, C> Deref for Scope<'a, C> {
    type Target = C;
    fn deref(&self) -> &C {
        self.ctx
    }
}

impl<'a, C> DerefMut for Scope<'a, C> {
    fn deref_mut(&mut self) -> &mut C {
        self.ctx
    }
}

impl<'a, C> Scope<'a, C> {
    pub fn create_future<T:Sized>(&mut self) -> (Port<T>, Future<T>) {
        pair(self.token, self.channel)
    }
}

pub fn scope<'x, C, L:LoopApi>(token: Token, ctx: &'x mut C,
    channel: &'x mut Sender<Notify>, loop_api: &'x mut L)
    -> Scope<'x, C>
{
    Scope {
        token: token,
        ctx: ctx,
        channel: channel,
        loop_api: loop_api,
    }
}
