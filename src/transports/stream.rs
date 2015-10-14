use std::marker::PhantomData;

use netbuf::Buf;
use time::SteadyTime;
use mio::{EventSet, PollOpt};

use super::StreamSocket as Socket;
use handler::{Registrator};
use {Async, EventMachine, BaseMachine};

pub struct Timeout(pub SteadyTime);

struct Inner<S: Socket> {
    socket: S,
    inbuf: Buf,
    outbuf: Buf,
    writable: bool,
    readable: bool,
}

pub struct Stream<C, S: Socket, P: Protocol<S, C>>
    (Inner<S>, P, PhantomData<*mut C>);

impl<C, S: Socket, P: Protocol<S, C>> BaseMachine for Stream<C, S, P> {
    type Message = P::Message;
    type Value = Timeout;
    fn message(self, msg: Self::Message) -> Async<Self, Self::Value> {
        let Stream(inn, proto, phan) = self;
        proto.message(msg).map(|p| Stream(inn, p, phan))
    }
}

impl<C, S: Socket, P: Protocol<S, C>> EventMachine<C> for Stream<C, S, P> {
    fn ready(self, events: EventSet, context: &mut C)
        -> Async<Self, Option<Self>>
    {
        unimplemented!();
    }

    fn register(&mut self, reg: &mut Registrator) {
        reg.register(&self.0.socket, EventSet::all(), PollOpt::edge());
    }

    fn timeout(&mut self) -> Async<Self, Option<Self>> {
        unimplemented!();
    }
}

pub trait Protocol<S, C>: BaseMachine<Value=Timeout> {
}
