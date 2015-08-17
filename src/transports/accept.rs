use std::io::Error;
use std::marker::PhantomData;

use mio::TryAccept;
use mio::{EventSet, Handler, Token, EventLoop, PollOpt, Evented};

use context::AsyncContext;
use {EventMachine};

pub enum Serve<S:TryAccept+Send, M, C>
    where M: Init<S::Output, C>, S: Evented
{
    Accept(S, PhantomData<*const C>),
    Connection(M),
}

unsafe impl<S:TryAccept+Send, M, C> Send for Serve<S, M, C>
    where M: Init<S::Output, C>, S: Evented {}

pub trait Init<S, C> {
    fn accept(conn: S, ctx: &mut C) -> Self;
}

impl<S, T, C, M> EventMachine<C> for Serve<S, M, C>
    where M: Init<T, C>,
          M: EventMachine<C>,
          S: Evented,
          S: TryAccept<Output=T>+Send,
          C: AsyncContext<Serve<S, M, C>>
{
    fn ready(self, evset: EventSet, context: &mut C) -> Option<Self> {
        use self::Serve::*;
        match self {
            Accept(sock, _) => {
                match sock.accept() {
                    Ok(Some(child)) => {
                        let conn = Connection(Init::accept(child, context));
                        context.async_add_machine(conn)
                        .map_err(|child| child.abort())
                        .ok();
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("Error on socket accept: {}", e);
                    }
                }
                Some(Accept(sock, PhantomData))
            }
            Connection(c) => c.ready(evset, context).map(Connection),
        }
    }

    fn register<H:Handler>(&mut self, tok: Token, eloop: &mut EventLoop<H>)
        -> Result<(), Error>
    {
        use self::Serve::*;
        match self {
            &mut Accept(ref sock, _) => {
                eloop.register_opt(sock, tok,
                    EventSet::readable(), PollOpt::level())
            }
            &mut Connection(ref mut c) => c.register(tok, eloop),
        }
    }
}

impl<S, T, C, M> Serve<S, M, C>
    where M: Init<T, C>,
          M: EventMachine<C>,
          S: Evented,
          S: TryAccept<Output=T>+Send,
          C: AsyncContext<Serve<S, M, C>>
{
    pub fn new(sock: S) -> Self {
        Serve::Accept(sock, PhantomData)
    }
}
