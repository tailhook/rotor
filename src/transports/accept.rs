use std::io::Error;
use std::marker::PhantomData;

use mio::TryAccept;
use mio::{EventSet, Handler, PollOpt, Evented};
use mio::{Timeout, TimerError};

use {EventMachine, Scope};
use handler::Abort::MachineAddError;

pub enum Serve<S, M, Ctx>
    where
        M: Init<S::Output, Ctx>, M: EventMachine<Ctx>, M: Send,
        S: TryAccept+Send, S: Evented,
{
    Accept(S, PhantomData<*const Ctx>),
    Connection(M),
}

unsafe impl<S:TryAccept+Send, M, Ctx> Send for Serve<S, M, Ctx>
    where
        M: Init<S::Output, Ctx>, M: EventMachine<Ctx>, M: Send,
        S: TryAccept+Send, S: Evented,
{}

pub trait Init<T, C>: EventMachine<C> {
    fn accept<'x, S>(conn: T, context: &mut C, scope: &mut S)
        -> Self
        where S: 'x, S: Scope<Self, Self::Timeout>;
}

struct ScopeProxy<'a, S: 'a, A, C>(&'a mut S, PhantomData<*const (A, C)>);

impl<'a, M, T, S, A, C> Scope<M, T> for ScopeProxy<'a, S, A, C>
    where S: Scope<Serve<A, M, C>, T> + 'a,
          A: TryAccept+Send, A: Evented,
          M: Init<A::Output, C>,
{
    fn async_add_machine(&mut self, m: M) -> Result<(), M> {
        self.0.async_add_machine(Serve::Connection(m))
        .map_err(|x| if let Serve::Connection(c) = x {
            c
        } else {
            unreachable!();
        })
    }
    fn add_timeout_ms(&mut self, delay: u64, t: T)
        -> Result<Timeout, TimerError>
    {
        self.0.add_timeout_ms(delay, t)
    }
    fn clear_timeout(&mut self, timeout: Timeout) -> bool {
        self.0.clear_timeout(timeout)
    }
    fn register<E: ?Sized>(&mut self, io: &E, interest: EventSet, opt: PollOpt)
        -> Result<(), Error>
        where E: Evented
    {
        self.0.register(io, interest, opt)
    }
}

impl<S, M, Ctx> EventMachine<Ctx> for Serve<S, M, Ctx>
    where M: Init<S::Output, Ctx>,
          M: EventMachine<Ctx>,
          S: Evented,
          S: TryAccept + Send,
{
    type Timeout = M::Timeout;
    fn ready<'x, Sc>(self, evset: EventSet, context: &mut Ctx, scope: &mut Sc)
        -> Option<Self>
        where Sc: 'x, Sc: Scope<Self, Self::Timeout>
    {
        use self::Serve::*;
        match self {
            Accept(sock, _) => {
                match sock.accept() {
                    Ok(Some(child)) => {
                        let conm: M = <M as Init<_, _>>::accept(child, context,
                            &mut ScopeProxy(scope, PhantomData));
                        let conn: Serve<S, M, Ctx> = Connection(conm);
                        scope.async_add_machine(conn)
                        .map_err(|child|
                            child.abort(MachineAddError, context, scope))
                        .ok();
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("Error on socket accept: {}", e);
                    }
                }
                Some(Accept(sock, PhantomData))
            }
            Connection(c) => c.ready(evset, context,
                &mut ScopeProxy(scope, PhantomData))
                .map(Connection),
        }
    }
    fn register<'x, Sc>(&mut self, scope: &mut Sc)
        -> Result<(), Error>
        where Sc: 'x, Sc: Scope<Self, Self::Timeout>
    {
        use self::Serve::*;
        match self {
            &mut Accept(ref mut s, _)
            => scope.register(s, EventSet::readable(), PollOpt::level()),
            &mut Connection(ref mut c)
            => c.register(&mut ScopeProxy(scope, PhantomData)),
        }
    }
}

impl<S, T, M, Ctx> Serve<S, M, Ctx>
    where M: Init<T, Ctx>,
          M: EventMachine<Ctx>,
          S: Evented,
          S: TryAccept<Output=T>+Send,
{
    pub fn new(sock: S) -> Self {
        Serve::Accept(sock, PhantomData)
    }
}
