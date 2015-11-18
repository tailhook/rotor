use std::cmp::max;
use std::ops::{Deref, DerefMut};

use time::SteadyTime;

use mio::{self, EventLoop, Token, EventSet, Evented, PollOpt};
use mio::util::Slab;

use {Async};


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Abort {
    NoSlabSpace,
    RegisterFailed,
    MachineAddError,
}

pub enum Timeo {
    Fsm(Token),
}

pub enum Notify {
    Fsm(Token),
}

pub struct Cell<M:Sized>(M, Option<(SteadyTime, mio::Timeout)>);

pub struct Scope<'a, C:Sized+'a>(Token, &'a mut C);

pub struct Handler<Ctx, M>
    where M: EventMachine<Ctx>
{
    slab: Slab<Cell<M>>,
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

impl<'a, C> Deref for Scope<'a, C> {
    type Target = C;
    fn deref(&self) -> &C {
        self.1
    }
}

impl<'a, C> DerefMut for Scope<'a, C> {
    fn deref_mut(&mut self) -> &mut C {
        self.1
    }
}

impl<'a, H> Registrator for Reg<'a, H>
    where H: mio::Handler + 'a, H::Timeout: 'a, H::Message: 'a
{
    fn register(&mut self, io: &Evented, interest: EventSet, opt: PollOpt)
    {
        self.eloop.register_opt(io, self.token, interest, opt).unwrap();
    }
}

pub trait EventMachine<C>: Sized {
    /// Socket readiness notification
    fn ready(self, events: EventSet, scope: &mut Scope<C>)
        -> Async<Self, Option<Self>>;

    /// Gives socket a chance to register in event loop
    fn register(self, reg: &mut Registrator) -> Async<Self, ()>;

    /// Timeout happened
    fn timeout(self, scope: &mut Scope<C>) -> Async<Self, Option<Self>>;

    /// Message received
    fn wakeup(self, scope: &mut Scope<C>) -> Async<Self, Option<Self>>;
}

impl<C, M> Handler<C, M>
    where M: EventMachine<C>,
{
    pub fn new(context: C, _eloop: &mut EventLoop<Handler<C, M>>)
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

fn replacement<M, C, R>(ares: Async<M, R>, eloop: &mut EventLoop<Handler<C, M>>,
    token: Token, old_timer: Option<(SteadyTime, mio::Timeout)>)
    -> (Option<Cell<M>>, Option<R>)
    where M:Sized, M:EventMachine<C>, R:Sized
{
    use async::Async::*;
    match ares {
        Continue(m, result) => {
            if let Some((_, ticket)) = old_timer {
                eloop.clear_timeout(ticket);
            }
            (Some(Cell(m, None)), Some(result))
        }
        Stop => (None, None),
        Timeout(m, deadline) => {
            let ticket = match old_timer {
                Some((dl, t)) if dl == deadline => Some(t),
                Some((_, ticket)) => {
                    eloop.clear_timeout(ticket);
                    None
                }
                None => None,
            };
            let ticket = ticket.unwrap_or_else(|| {
                let left = deadline - SteadyTime::now();
                eloop.timeout_ms(
                        Timeo::Fsm(token),
                        max(left.num_milliseconds(), 0) as u64,
                    ).ok().expect("No more timer slots?")
            });
            (Some(Cell(m, Some((deadline, ticket)))), None)
        }
    }
}

impl<'a, Ctx, M> Handler<Ctx, M>
    where M: EventMachine<Ctx>
{
    pub fn add_root(&mut self, eloop: &mut EventLoop<Self>, m: M) {
        match self.slab.insert(Cell(m, None)) {
            Ok(tok) => {
                self.slab.replace_with(tok, |Cell(m, timer)| {
                    let mach = m.register(&mut Reg {
                        eloop: eloop,
                        token: tok
                        });
                    replacement(mach, eloop, tok, timer).0
                }).unwrap(); // just inserted so must work
            }
            Err(_) => {
                unimplemented!();
            }
        }
    }

    fn action_loop<'x, F>(&mut self, token: Token,
        eloop: &'x mut EventLoop<Self>, fun: F)
        where F: Fn(M, &mut Ctx) -> Async<M, Option<M>>,
    {
        let ref mut ctx = self.context;
        loop {
            let mut new_machine = None;
            self.slab.replace_with(token, |Cell(m, timer)| {
                let mach = fun(m, ctx);
                let (cell, res) = replacement(mach, eloop, token, timer);
                new_machine = res.and_then(|r| r);
                cell
            }).ok();  // Spurious events are ok in mio
            if let Some(new) = new_machine {
                match self.slab.insert(Cell(new, None)) {
                    Ok(tok) => {
                        self.slab.replace_with(tok, |Cell(m, timer)| {
                            let mach = m.register(&mut Reg {
                                eloop: eloop,
                                token: tok
                                });
                            replacement(mach, eloop, tok, timer).0
                        }).unwrap(); // just inserted so must work
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
}

impl<Ctx, M> mio::Handler for Handler<Ctx, M>
    where M: EventMachine<Ctx>
{
    type Message = Notify;
    type Timeout = Timeo;
    fn ready<'x>(&mut self, eloop: &'x mut EventLoop<Self>,
        token: Token, events: EventSet)
    {
        self.action_loop(token, eloop,
            |m, ctx| m.ready(events, &mut Scope(token, ctx)));
    }

    fn notify(&mut self, eloop: &mut EventLoop<Self>, msg: Notify) {
        match msg {
            Notify::Fsm(token) => {
                self.action_loop(token, eloop,
                    |m, ctx| m.wakeup(&mut Scope(token, ctx)));
            }
        }
    }

    fn timeout(&mut self, eloop: &mut EventLoop<Self>, timeo: Timeo) {
        match timeo {
            Timeo::Fsm(token) => {
                self.action_loop(token, eloop,
                    |m, ctx| m.wakeup(&mut Scope(token, ctx)));
            }
        }

    }
}

