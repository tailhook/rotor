use std::error::Error;
use std::mem::{forget, zeroed, swap};

use mio::EventSet;

use {Machine, Scope, Response, NoSlabSpace};


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

impl<X, A: Machine<X>, B:Machine<X>> Machine<X> for Compose2<A, B> {
    type Seed = Compose2Seed<A::Seed, B::Seed>;

    fn create(seed: Self::Seed, scope: &mut Scope<X>)
        -> Result<Self, Box<Error>> {
        use Compose2::*;
        use self::Compose2Seed::*;
        match seed {
            As(s) => A::create(s, scope).map(A).map_err(|mut e| {
                if e.is::<NoSlabSpace<A::Seed>>() {
                    let mut s: NoSlabSpace<A::Seed> = unsafe { zeroed() };
                    swap(&mut s,
                        e.downcast_mut::<NoSlabSpace<A::Seed>>().unwrap());
                    forget(e);
                    Box::new(NoSlabSpace(As::<_, B::Seed>(s.0))) as Box<Error>
                } else {
                    e
                }
            }),
            Bs(s) => B::create(s, scope).map(B).map_err(|mut e| {
                if e.is::<NoSlabSpace<B::Seed>>() {
                    let mut s: NoSlabSpace<B::Seed> = unsafe { zeroed() };
                    swap(&mut s,
                        e.downcast_mut::<NoSlabSpace<B::Seed>>().unwrap());
                    forget(e);
                    Box::new(NoSlabSpace(Bs::<A::Seed, _>(s.0))) as Box<Error>
                } else {
                    e
                }
            }),
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
