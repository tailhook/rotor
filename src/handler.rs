use mio::{self, EventLoop, Token, EventSet, Evented, Sender};
use mio::util::Slab;

use scope::scope;
use {Async, Scope, Notify, BaseMachine};


pub enum Timeo {
    Fsm(Token),
}

pub struct Handler<Ctx, M>
    where M: EventMachine<Ctx>
{
    slab: Slab<M>,
    context: Ctx,
    channel: Sender<Notify>,
}

pub trait EventMachine<C>: BaseMachine<C, Value=Self, State=()>
{
    /// Socket readiness notification
    fn ready(self, events: EventSet, scope: &mut Scope<C>)
        -> Async<Self, Self, ()>;

    /// Gives socket a chance to register in event loop
    fn register(self, scope: &mut Scope<C>) -> Async<Self, Self, ()>;
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
            channel: eloop.channel(),
        }
    }
}

impl<'a, Ctx, M> Handler<Ctx, M>
    where M: EventMachine<Ctx>
{
    pub fn add_root(&mut self, eloop: &mut EventLoop<Self>, m: M) {
        // TODO(tailhook) give a chance to register
        match self.slab.insert(m) {
            Ok(token) => {
                machine_loop(self, eloop, token, |m, scope| m.register(scope))
            }
            Err(_) => {
                panic!("Can't add root");
            }
        }
    }

}

struct AResult<M:Sized> {
    machine: Option<M>,
    value: Option<M>,
    next_iter: bool,
}

impl<M:Sized> AResult<M> {
    fn from(val: Async<M, M, ()>) -> AResult<M>
    {
        match val {
            Async::Send(m, v) => AResult {
                machine: Some(m),
                value: Some(v),
                next_iter: true,
            },
            Async::Yield(m, ()) => AResult {
                machine: Some(m),
                value: None,
                next_iter: false,
            },
            Async::Return(m, v, ()) => AResult {
                machine: Some(m),
                value: Some(v),
                next_iter: false,
            },
            Async::Ignore(m) => AResult {
                machine: Some(m),
                value: None,
                next_iter: false,
            },
            Async::Stop => AResult {
                machine: None,
                value: None,
                next_iter: false,
            },
        }
    }
}

fn machine_loop<C, M, F>(handler: &mut Handler<C, M>,
    eloop: &mut EventLoop<Handler<C, M>>, token: Token, fun: F)
    where M: EventMachine<C>,
          F: FnOnce(M, &mut Scope<C>) -> Async<M, M, ()>
{
    let mut next_iter = true;
    let mut nmachine = None;
    {
        let ref mut scope = scope(token, &mut handler.context,
            &mut handler.channel, eloop);
        handler.slab.replace_with(token, |m| {
            let ar = AResult::from(fun(m, scope));
            next_iter = ar.next_iter;
            nmachine = ar.value;
            ar.machine
        }).ok();  // Spurious events are ok in mio
    }
    if let Some(m) = nmachine {
        handler.add_root(eloop, m);
    }
    while next_iter {
        let mut nmachine = None;
        {
            let ref mut scope = scope(token, &mut handler.context,
                &mut handler.channel, eloop);
            handler.slab.replace_with(token, |m| {
                let ar = AResult::from(m.wakeup(scope));
                next_iter = ar.next_iter;
                nmachine = ar.value;
                ar.machine
            }).unwrap();  // We know that machine is here
        }
        if let Some(m) = nmachine {
            handler.add_root(eloop, m);
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
        machine_loop(self, eloop, token, |m, scope| { m.ready(events, scope) })
    }

    fn notify(&mut self, eloop: &mut EventLoop<Self>, msg: Notify) {
        match msg {
            Notify::Fsm(token) => {
                machine_loop(self, eloop, token,
                    |m, scope| { m.wakeup(scope) })
            }
        }
    }

    fn timeout(&mut self, eloop: &mut EventLoop<Self>, timeo: Timeo) {
        match timeo {
            Timeo::Fsm(token) => {
                machine_loop(self, eloop, token,
                    |m, scope| { m.timeout(scope) })
            }
        }
    }
}

