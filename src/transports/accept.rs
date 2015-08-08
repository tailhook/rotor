use std::io::Error;
use mio::{TryAccept, EventSet, Handler, Token, EventLoop, PollOpt, Evented};

use {EventMachine};

pub enum Serve<S:TryAccept+Send, M: EventMachine>
    where M: Init<S::Output>, S: Evented
{
    Accept(S),
    Connection(M),
}

pub trait Init<C> {
    fn accept(c: C) -> Self;
}

impl<S: TryAccept+Send, M: EventMachine> EventMachine for Serve<S, M>
    where M: Init<S::Output>, S: Evented
{
    fn ready(self, evset: EventSet) -> Option<Self> {
        use self::Serve::*;
        match self {
            Accept(sock) => {
                match sock.accept() {
                    Ok(Some(child)) => {
                        // TODO(tailhook) create state machine and send
                        let m: M = Init::accept(child);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        error!("Error on socket accept: {}", e);
                    }
                }
                Some(Accept(sock))
            }
            Connection(c) => c.ready(evset).map(Connection),
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
