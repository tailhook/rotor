use std::marker::PhantomData;

use mio::EventSet;

use handler::Registrator;
use {Async, Scope, EventMachine};



struct TimeMachine<C:Sized, M:Timer<C>>(M, PhantomData<*const C>);

/// Experimental trait to simplify global timers
pub trait Timer<C:Sized>: Sized {
    fn timeout(self, scope: &mut Scope<C>) -> Async<Self, Option<Self>>;
}

impl<C, M:Timer<C>> EventMachine<C> for TimeMachine<C, M> {
    fn ready(self, evset: EventSet, scope: &mut Scope<C>)
        -> Async<Self, Option<Self>>
    {
        unreachable!();
    }
    fn register(self, reg: &mut Registrator) -> Async<Self, ()> {
        Async::Continue(self, ())
    }
    fn timeout(self, scope: &mut Scope<C>) -> Async<Self, Option<Self>> {
        Timer::timeout(self.0, scope).wrap(|x| TimeMachine(x, PhantomData))
    }

    fn wakeup(self, scope: &mut Scope<C>) -> Async<Self, Option<Self>> {
        Async::Continue(self, None)
    }
}
