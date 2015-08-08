use std::io::Error;

use mio::{TryAccept, EventSet, Handler, Token, EventLoop, PollOpt, Evented};

use context::AsyncContext;
use {EventMachine};

pub enum Serve<S:TryAccept+Send, M>
    where M: Init<S::Output>, S: Evented
{
    Accept(S),
    Connection(M),
}

unsafe impl<S:TryAccept+Send, M> Send for Serve<S, M>
    where M: Init<S::Output>, S: Evented {}

pub trait Init<C> {
    fn accept(conn: C) -> Self;
}

impl<S, T, C, M> EventMachine<C> for Serve<S, M>
    where M: Init<T>,
          M: EventMachine<C>,
          S: Evented,
          S: TryAccept<Output=T>+Send,
          C: AsyncContext<Serve<S, M>>
{
    fn ready(self, evset: EventSet, context: &mut C) -> Option<Self> {
        use self::Serve::*;
        match self {
            Accept(sock) => {
                match sock.accept() {
                    Ok(Some(child)) => {
                        context.async_add_machine(
                            Connection(Init::accept(child)))
                        .map_err(|child| child.abort())
                        .ok();
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("Error on socket accept: {}", e);
                    }
                }
                Some(Accept(sock))
            }
            Connection(c) => c.ready(evset, context).map(Connection),
        }
    }

    fn register<H:Handler>(&mut self, tok: Token, eloop: &mut EventLoop<H>)
        -> Result<(), Error>
    {
        use self::Serve::*;
        match self {
            &mut Accept(ref sock) => {
                eloop.register_opt(sock, tok,
                    EventSet::readable(), PollOpt::level())
            }
            &mut Connection(ref mut c) => c.register(tok, eloop),
        }
    }
}
