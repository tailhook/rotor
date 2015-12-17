use mio::EventSet;
use {Machine, Creator, Scope, Response};


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

pub enum Compose2Creator<A:Sized, B:Sized> {
    Ac(A),
    Bc(B),
}

pub enum Compose2CreatorError<A:Sized, B:Sized> {
    Ae(A),
    Be(B),
}

impl<X, A: Machine<X>, B:Machine<X>> Machine<X> for Compose2<A, B> {
    type Creator = Compose2Creator<A::Creator, B::Creator>;

    fn ready(self, events: EventSet, scope: &mut Scope<X>)
        -> Response<Self, Self::Creator>
    {
        use Compose2::*;
        use self::Compose2Creator::*;
        match self {
            A(m) => { m.ready(events, scope).map(A, Ac) }
            B(m) => { m.ready(events, scope).map(B, Bc) }
        }
    }
    fn spawned(self, scope: &mut Scope<X>) -> Response<Self, Self::Creator>
    {
        use Compose2::*;
        use self::Compose2Creator::*;
        match self {
            A(m) => { m.spawned(scope).map(A, Ac) }
            B(m) => { m.spawned(scope).map(B, Bc) }
        }
    }
    fn timeout(self, scope: &mut Scope<X>) -> Response<Self, Self::Creator> {
        use Compose2::*;
        use self::Compose2Creator::*;
        match self {
            A(m) => { m.timeout(scope).map(A, Ac) }
            B(m) => { m.timeout(scope).map(B, Bc) }
        }
    }
    fn wakeup(self, scope: &mut Scope<X>) -> Response<Self, Self::Creator> {
        use Compose2::*;
        use self::Compose2Creator::*;
        match self {
            A(m) => { m.wakeup(scope).map(A, Ac) }
            B(m) => { m.wakeup(scope).map(B, Bc) }
        }
    }
}

impl<X, A: Creator<X>, B:Creator<X>> Creator<X> for Compose2Creator<A, B> {
    type Machine = Compose2<A::Machine, B::Machine>;
    type Error = Compose2CreatorError<A::Error, B::Error>;
    fn create(self, scope: &mut Scope<X>)
        -> Result<Self::Machine, Self::Error>
    {
        use self::Compose2::*;
        use self::Compose2Creator::*;
        use self::Compose2CreatorError::*;
        match self {
            Ac(c) => c.create(scope).map(A).map_err(Ae),
            Bc(c) => c.create(scope).map(B).map_err(Be),
        }
    }
}
