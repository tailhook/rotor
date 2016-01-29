use std::error::Error;

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

pub enum Compose2Seed<A:Sized, B:Sized> {
    As(A),
    Bs(B),
}

impl<X, AA, BB> Machine for Compose2<AA, BB>
    where AA: Machine<Context=X>, BB:Machine<Context=X>
{
    type Context = X;
    type Seed = Compose2Seed<AA::Seed, BB::Seed>;

    fn create(seed: Self::Seed, scope: &mut Scope<X>)
        -> Result<Self, Box<Error>>
    {
        use Compose2::*;
        use self::Compose2Seed::*;
        match seed {
            As(s) => AA::create(s, scope).map(A),
            Bs(s) => BB::create(s, scope).map(B),
        }
    }
    fn ready(self, events: EventSet, scope: &mut Scope<X>)
        -> Response<Self, Self::Seed>
    {
        use Compose2::*;
        use self::Compose2Seed::*;
        match self {
            A(m) => { m.ready(events, scope).map(A, As) }
            B(m) => { m.ready(events, scope).map(B, Bs) }
        }
    }
    fn spawned(self, scope: &mut Scope<X>) -> Response<Self, Self::Seed>
    {
        use Compose2::*;
        use self::Compose2Seed::*;
        match self {
            A(m) => { m.spawned(scope).map(A, As) }
            B(m) => { m.spawned(scope).map(B, Bs) }
        }
    }
    fn timeout(self, scope: &mut Scope<X>) -> Response<Self, Self::Seed> {
        use Compose2::*;
        use self::Compose2Seed::*;
        match self {
            A(m) => { m.timeout(scope).map(A, As) }
            B(m) => { m.timeout(scope).map(B, Bs) }
        }
    }
    fn wakeup(self, scope: &mut Scope<X>) -> Response<Self, Self::Seed> {
        use Compose2::*;
        use self::Compose2Seed::*;
        match self {
            A(m) => { m.wakeup(scope).map(A, As) }
            B(m) => { m.wakeup(scope).map(B, Bs) }
        }
    }
}
