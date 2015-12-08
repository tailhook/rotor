use std::marker::PhantomData;
use std::io;
use std::io::ErrorKind::{WouldBlock, Interrupted};

use netbuf::Buf;
use time::SteadyTime;
use mio::{EventSet, PollOpt};
use void::Void;

use super::StreamSocket as Socket;
use super::accept::Init;
use {Async, BaseMachine, EventMachine, Scope};

pub struct Timeout(pub SteadyTime);

struct Inner<S: Socket> {
    socket: S,
    inbuf: Buf,
    outbuf: Buf,
    writable: bool,
    readable: bool,
}

pub struct Stream<C, S: Socket, P: Protocol<C>>
    (Inner<S>, P, PhantomData<*mut C>);

pub struct Transport<'a> {
    inbuf: &'a mut Buf,
    outbuf: &'a mut Buf,
}


impl<S: Socket> Inner<S> {
    fn transport(&mut self) -> Transport {
        Transport {
            inbuf: &mut self.inbuf,
            outbuf: &mut self.outbuf,
        }
    }
}

impl<C, S: Socket, P: Protocol<C>> Init<S, C> for Stream<C, S, P> {
    fn accept(mut conn: S) -> Option<Self>
    {
        let protocol = match Protocol::accepted(&mut conn) {
            Some(x) => x,
            None => return None
        };

        Some(Stream(Inner {
            socket: conn,
            inbuf: Buf::new(),
            outbuf: Buf::new(),
            readable: false,
            writable: true,   // Accepted socket is immediately writable
        }, protocol, PhantomData))
    }
}

impl<C, S: Socket, P: Protocol<C>> EventMachine<C> for Stream<C, S, P> {
    fn ready(self, evset: EventSet, scope: &mut Scope<C>)
        -> Async<Self, Self, ()>
    {
        let Stream(mut stream, fsm, _) = self;
        let mut monad = Async::Yield(fsm, ());
        if evset.is_writable() && stream.outbuf.len() > 0 {
            stream.writable = true;
            while stream.outbuf.len() > 0 {
                match stream.outbuf.write_to(&mut stream.socket) {
                    Ok(0) => { // Connection closed
                        monad.done(|fsm| fsm.eof_received(scope));
                        return Async::Stop;
                    }
                    Ok(_) => {
                        monad = async_try!(monad.map(|f| {
                            f.data_transferred(
                                &mut stream.transport(), scope)
                        }));
                    }
                    Err(ref e) if e.kind() == WouldBlock => {
                        stream.writable = false;
                        break;
                    }
                    Err(ref e) if e.kind() == Interrupted =>  { continue; }
                    Err(e) => {
                        monad.done(|fsm| fsm.error_happened(e, scope));
                        return Async::Stop;
                    }
                }
            }
        }
        if evset.is_readable() {
            stream.readable = true;
            loop {
                match stream.inbuf.read_from(&mut stream.socket) {
                    Ok(0) => { // Connection closed
                        monad.done(|fsm| fsm.eof_received(scope));
                        return Async::Stop;
                    }
                    Ok(_) => {
                        monad = async_try!(monad.and_then(|f| {
                            f.data_received(
                                &mut stream.transport(), scope)
                        }));
                    }
                    Err(ref e) if e.kind() == WouldBlock => {
                        stream.readable = false;
                        break;
                    }
                    Err(ref e) if e.kind() == Interrupted =>  { continue; }
                    Err(e) => {
                        monad.done(|fsm| fsm.error_happened(e, scope));
                        return Async::Stop;
                    }
                }
            }
        }
        if stream.writable && stream.outbuf.len() > 0 {
            while stream.outbuf.len() > 0 {
                match stream.outbuf.write_to(&mut stream.socket) {
                    Ok(0) => { // Connection closed
                        monad.done(|fsm| fsm.eof_received(scope));
                        return Async::Stop;
                    }
                    Ok(_) => {
                        monad = async_try!(monad.and_then(|f| {
                            f.data_transferred(
                                &mut stream.transport(), scope)
                        }));
                    }
                    Err(ref e) if e.kind() == WouldBlock => {
                        stream.writable = false;
                        break;
                    }
                    Err(ref e) if e.kind() == Interrupted =>  { continue; }
                    Err(e) => {
                        monad.done(|fsm| fsm.error_happened(e, scope));
                        return Async::Stop;
                    }
                }
            }
        }
        monad
        .map(|fsm| Stream(stream, fsm, PhantomData))
        .map_result(|()| None)
    }

    fn register(self, scope: &mut Scope<C>) -> Async<Self, Self, ()> {
        scope.register(&self.0.socket, EventSet::readable(), PollOpt::level());
        Async::Continue(self, ())
    }
}

impl<C, S: Socket, P: Protocol<C>> BaseMachine<C> for Stream<C, S, P> {
    type Value = Self;
    type State = ();
    fn timeout(self, scope: &mut Scope<C>) -> Async<Self, Self, ()> {
        let Stream(stream, fsm, _) = self;
        fsm.timeout(scope)
        .map(|fsm| Stream(stream, fsm, PhantomData))
    }

    fn wakeup(self, scope: &mut Scope<C>) -> Async<Self, Self, ()> {
        let Stream(stream, fsm, _) = self;
        fsm.wakeup(scope)
        .map(|fsm| Stream(stream, fsm, PhantomData))
    }
}

pub trait Protocol<C>: BaseMachine<C, Value=Void, State=()> {
    fn accepted<S: Socket>(conn: &mut S)
        -> Option<Self>;
    fn data_received(self, trans: &mut Transport, scope: &mut Scope<C>)
        -> Async<Self, Void, ()>;
    fn data_transferred(self, _trans: &mut Transport, _scope: &mut Scope<C>)
        -> Async<Self, Void, ()> {
        Async::Continue(self, ())
    }
    // TODO(tailhook) some error object should be here
    fn error_happened(self, _err: io::Error, _scope: &mut Scope<C>) {}
    fn eof_received(self, _scope: &mut Scope<C>) {}
}

impl<'a> Transport<'a> {
    pub fn input<'x>(&'x mut self) -> &'x mut Buf {
        self.inbuf
    }
    pub fn output<'x>(&'x mut self) -> &'x mut Buf {
        self.outbuf
    }
}
