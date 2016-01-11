use std::error::Error;

use mio;

use scope::{scope, Scope};
use {Response, Handler, Machine, Notify};

pub struct Stub;

impl<C:Sized> Machine<C> for Stub {
    type Seed = ();
    fn create(_seed: Self::Seed, _scope: &mut Scope<C>)
        -> Result<Self, Box<Error>>
    { unreachable!() }
    fn ready(self, _events: mio::EventSet, _scope: &mut Scope<C>)
        -> Response<Self, Self::Seed>
    { unreachable!() }
    fn spawned(self, _scope: &mut Scope<C>)
        -> Response<Self, Self::Seed>
    { unreachable!() }
    fn spawn_error(self, _scope: &mut Scope<C>, _error: Box<Error>)
        -> Option<Self>
    { unreachable!() }
    fn timeout(self, _scope: &mut Scope<C>) -> Response<Self, Self::Seed>
    { unreachable!() }
    fn wakeup(self, _scope: &mut Scope<C>) -> Response<Self, Self::Seed>
    { unreachable!() }
}

pub struct Loop<C> {
    eloop: mio::EventLoop<Handler<C, Stub>>,
    chan: mio::Sender<Notify>,
    pub ctx: C,
}


impl<C:Sized> Loop<C> {
    pub fn new(ctx: C) -> Loop<C> {
        let eloop = mio::EventLoop::new().unwrap();
        Loop { chan: eloop.channel(), eloop: eloop, ctx: ctx }
    }
    pub fn scope<'x>(&'x mut self, n: usize) -> Scope<'x, C> {
        return scope(mio::Token(n), &mut self.ctx,
            &mut self.chan, &mut self.eloop);
    }
}
