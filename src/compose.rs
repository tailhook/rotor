use mio::EventSet;
use {Machine, Scope, Response};


/// Composes two state machines
///
/// Used to "mount" two different application into single main loop, or to
/// use multiple protocols simultaneously. Can be nested to any level.
///
/// We will probably implement n > 2 composition later, for effeciency
/// reasons.
pub enum Compose2<A:Sized, B:Sized> {
    A(A),
    B(B),
}

impl<X, A: Machine<X>, B:Machine<X>> Machine<X> for Compose2<A, B> {
    fn register(self, scope: &mut Scope<X>) -> Response<Self> {
        use Compose2::*;
        match self {
            A(m) => { m.register(scope).map(A, A) }
            B(m) => { m.register(scope).map(B, B) }
        }
    }
    fn ready(self, events: EventSet, scope: &mut Scope<X>)
        -> Response<Self>
    {
        use Compose2::*;
        match self {
            A(m) => { m.ready(events, scope).map(A, A) }
            B(m) => { m.ready(events, scope).map(B, B) }
        }
    }
    fn spawned(self, scope: &mut Scope<X>) -> Response<Self>
    {
        use Compose2::*;
        match self {
            A(m) => { m.spawned(scope).map(A, A) }
            B(m) => { m.spawned(scope).map(B, B) }
        }
    }
    fn timeout(self, scope: &mut Scope<X>) -> Response<Self> {
        use Compose2::*;
        match self {
            A(m) => { m.timeout(scope).map(A, A) }
            B(m) => { m.timeout(scope).map(B, B) }
        }
    }
    fn wakeup(self, scope: &mut Scope<X>) -> Response<Self> {
        use Compose2::*;
        match self {
            A(m) => { m.wakeup(scope).map(A, A) }
            B(m) => { m.wakeup(scope).map(B, B) }
        }
    }
}
