use std::marker::PhantomData;

use mio::TryAccept;
use mio::{EventSet, Handler, PollOpt, Evented};

use {Async, EventMachine};
use handler::Registrator;

pub enum Serve<S, M, C>
    where
        M: Init<S::Output, C>, M: EventMachine<C>,
        S: TryAccept, S: Evented,
{
    Accept(S, PhantomData<*const C>),
    Connection(M),
}

pub trait Init<T, C>: Sized {
    fn accept(conn: T, context: &mut C) -> Option<Self>;
}

impl<S, M, C> Serve<S, M, C>
    where M: Init<S::Output, C>, M: EventMachine<C>,
          S: TryAccept, S: Evented,
{
    pub fn new(sock: S) -> Self {
        Serve::Accept(sock, PhantomData)
    }
}


impl<C, S, M: EventMachine<C>> EventMachine<C> for Serve<S, M, C>
    where S: TryAccept, S: Evented, M: Init<S::Output, C>
{
    fn ready(self, evset: EventSet, context: &mut C)
        -> Async<Self, Option<Self>>
    {
        use self::Serve::*;
        match self {
            Accept(sock, _) => {
                let new_machine = match sock.accept() {
                    Ok(Some(child)) => {
                        <M as Init<_, _>>::accept(child, context)
                    }
                    Ok(None) => None,
                    Err(e) => {
                        error!("Error on socket accept: {}", e);
                        None
                    }
                };
                Async::Continue(Accept(sock, PhantomData),
                    new_machine.map(Connection))
            }
            Connection(c) => {
                c.ready(evset, context)
                    .map(Connection).map_result(|x| x.map(Connection))
            }
        }
    }
    fn register(self, reg: &mut Registrator) -> Async<Self, ()> {
        use self::Serve::*;
        match self {
            Accept(s, _) => {
                reg.register(&s, EventSet::readable(), PollOpt::level());
                Async::Continue(Accept(s, PhantomData), ())
            }
            Connection(c) => c.register(reg).map(Connection),
        }
    }
    fn timeout(self, context: &mut C) -> Async<Self, Option<Self>> {
        use self::Serve::*;
        match self {
            me @ Accept(_, _) => Async::Continue(me, None),
            Connection(c) => {
                c.timeout(context)
                    .map(Connection).map_result(|x| x.map(Connection))
            }
        }
    }

    fn wakeup(self, context: &mut C) -> Async<Self, Option<Self>> {
        use self::Serve::*;
        match self {
            me @ Accept(_, _) => Async::Continue(me, None),
            Connection(c) => {
                c.wakeup(context)
                    .map(Connection).map_result(|x| x.map(Connection))
            }
        }
    }
}
