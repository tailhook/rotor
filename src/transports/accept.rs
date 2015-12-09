use std::marker::PhantomData;

use mio::TryAccept;
use mio::{EventSet, Handler, PollOpt, Evented};

use {Async, BaseMachine, EventMachine, Scope};

pub enum Serve<C, S, M>
    where
        M: Init<S::Output, C>, M: EventMachine<C>,
        S: TryAccept, S: Evented,
{
    Accept(S, PhantomData<*const C>),
    Connection(M),
}

pub trait Init<T, C>: Sized {
    fn accept(conn: T) -> Option<Self>;
}

impl<S, M, C> Serve<C, S, M>
    where M: Init<S::Output, C>, M: EventMachine<C>,
          S: TryAccept, S: Evented,
{
    pub fn new(sock: S) -> Self {
        Serve::Accept(sock, PhantomData)
    }
}

impl<C, S, M: EventMachine<C>> EventMachine<C> for Serve<C, S, M>
    where S: TryAccept, S: Evented, M: Init<S::Output, C>
{
    fn ready(self, evset: EventSet, scope: &mut Scope<C>)
        -> Async<Self, Self, ()>
    {
        use self::Serve::*;
        match self {
            Accept(sock, _) => {
                match sock.accept() {
                    Ok(Some(child)) => {
                        if let Some(m) = <M as Init<_, _>>::accept(child) {
                            Async::Send(Accept(sock, PhantomData),
                                Connection(m))
                        } else {
                            Async::Ignore(Accept(sock, PhantomData))
                        }
                    }
                    Ok(None) => {
                        Async::Ignore(Accept(sock, PhantomData))
                    }
                    Err(e) => {
                        error!("Error on socket accept: {}", e);
                        Async::Ignore(Accept(sock, PhantomData))
                    }
                }
            }
            Connection(c) => {
                c.ready(evset, scope)
                    .map(Connection).map_result(Connection)
            }
        }
    }
    fn register(self, scope: &mut Scope<C>) -> Async<Self, Self, ()> {
        use self::Serve::*;
        match self {
            Accept(s, _) => {
                scope.register(&s, EventSet::readable(), PollOpt::level())
                    .unwrap(); // TODO(tailhook)
                Async::Yield(Accept(s, PhantomData), ())
            }
            Connection(c) => c.register(scope)
                .map(Connection).map_result(Connection),
        }
    }
}

impl<C, S, M: EventMachine<C>> BaseMachine<C> for Serve<C, S, M>
    where S: TryAccept, S: Evented, M: Init<S::Output, C>
{
    type Value = Self;
    type State = ();
    fn timeout(self, scope: &mut Scope<C>) -> Async<Self, Self, ()> {
        use self::Serve::*;
        match self {
            me @ Accept(_, _) => Async::Ignore(me),
            Connection(c) => {
                c.timeout(scope)
                    .map(Connection).map_result(Connection)
            }
        }
    }

    fn wakeup(self, scope: &mut Scope<C>) -> Async<Self, Self, ()> {
        use self::Serve::*;
        match self {
            Accept(sock, _) => {
                match sock.accept() {
                    Ok(Some(child)) => {
                        if let Some(m) = <M as Init<_, _>>::accept(child) {
                            Async::Send(Accept(sock, PhantomData),
                                Connection(m))
                        } else {
                            Async::Ignore(Accept(sock, PhantomData))
                        }
                    }
                    Ok(None) => {
                        Async::Ignore(Accept(sock, PhantomData))
                    }
                    Err(e) => {
                        error!("Error on socket accept: {}", e);
                        Async::Ignore(Accept(sock, PhantomData))
                    }
                }
            }
            Connection(c) => {
                c.wakeup(scope)
                    .map(Connection).map_result(Connection)
            }
        }
    }
}
