use std::io::Error;
use std::usize;

use time::SteadyTime;

use mio::{self, EventLoop, Token, EventSet, Evented, PollOpt};
use mio::util::Slab;
use mio::{Sender, TimerError};

use {Async, BaseMachine};


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Abort {
    NoSlabSpace,
    RegisterFailed,
    MachineAddError,
}

pub enum Notify<T> {
    NewMachine(T),
}

pub enum Timeo {
    Fsm(Token),
}

pub struct Handler<Ctx, M> {
    slab: Slab<M>,
    context: Ctx,
}

pub trait Registrator {
    fn register(&mut self, io: &Evented, interest: EventSet, opt: PollOpt);
}

struct Reg<'a, H>
    where H: mio::Handler + 'a, H::Timeout: 'a, H::Message: 'a
{
    eloop: &'a mut EventLoop<H>,
    token: Token,
}

impl<'a, H> Registrator for Reg<'a, H>
    where H: mio::Handler + 'a, H::Timeout: 'a, H::Message: 'a
{
    fn register(&mut self, io: &Evented, interest: EventSet, opt: PollOpt)
    {
        self.eloop.register_opt(io, self.token, interest, opt).unwrap();
    }
}

pub trait EventMachine<C>: BaseMachine {
    /// Socket readiness notification
    fn ready(self, events: EventSet, context: &mut C)
        -> Async<Self, Option<Self>>;

    /// Gives socket a chance to register in event loop
    fn register(&mut self, reg: &mut Registrator);

    /// Timeout happened
    fn timeout(&mut self) -> Async<Self, Option<Self>>;
}

impl<C, M> Handler<C, M>
    where M: EventMachine<C>,
{
    pub fn new(context: C, eloop: &mut EventLoop<Handler<C, M>>)
        -> Handler<C, M>
    {
        // TODO(tailhook) create default config from the ulimit data instead
        // of using real defaults
        Handler {
            slab: Slab::new(4096),
            context: context,
        }
    }
}

impl<'a, Ctx, M> mio::Handler for Handler<Ctx, M>
    where M: EventMachine<Ctx>
{
    type Message = (Token, M::Message);
    type Timeout = Timeo;
    fn ready<'x>(&mut self, eloop: &'x mut EventLoop<Self>,
        token: Token, events: EventSet)
    {
        use async::Async::*;
        let ref mut ctx = self.context;
        loop {
            let mut new_machine = None;
            self.slab.replace_with(token, |fsm| {
                match fsm.ready(events, ctx) {
                    Continue(m, new) => {
                        new_machine = new;
                        Some(m)
                    }
                    Stop => None,
                    Timeout(m, timeo) => {
                        Some(m)
                    }
                }
            }).ok();  // Spurious events are ok in mio
            if let Some(new) = new_machine {
                match self.slab.insert(new) {
                    Ok(tok) => {
                        self.slab.get_mut(tok).map(|fsm| {
                            fsm.register(&mut Reg {
                                eloop: eloop,
                                token: tok
                                });
                        });
                    }
                    Err(_) => {
                        unimplemented!();
                    }
                }
            } else {
                break;
            }
        }
    }

    fn notify(&mut self, eloop: &mut EventLoop<Self>, msg: Self::Message) {
        /*
        use self::Notify::*;
        let ref mut ctx = self.context;
        match msg {
            NewMachine(fsm) => {
                // This is so complex because of limitations of Slab
                match self.slab.insert(fsm) {
                    Ok(tok) => {
                        self.slab.replace_with(tok, |mut fsm| {
                            fsm.register(&mut Reg {
                                eloop: eloop,
                                token: tok
                                });
                            Some(fsm)
                        }).unwrap();
                    }
                    Err(fsm) => {
                        unimplemented!(); // Too many state machines"
                    }
                }
            }
        }
        */
        unimplemented!();
    }
}

