use mio::{self, EventLoop, Token, EventSet, Evented, Sender};
use mio::util::Slab;

use scope::scope;
use {Scope, Notify, Response};


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

pub trait EventMachine<C>: Sized {
    /// Socket readiness notification
    fn ready(self, events: EventSet, scope: &mut Scope<C>) -> Response<Self>;

    /// Called after spawn event
    fn spawned(self, scope: &mut Scope<C>) -> Response<Self>;

    /// Gives socket a chance to register in event loop
    fn register(self, scope: &mut Scope<C>) -> Response<Self>;

    /// Timeout happened
    fn timeout(self, scope: &mut Scope<C>) -> Response<Self>;

    /// Message received
    fn wakeup(self, scope: &mut Scope<C>) -> Response<Self>;
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

fn machine_loop<C, M, F>(handler: &mut Handler<C, M>,
    eloop: &mut EventLoop<Handler<C, M>>, token: Token, fun: F)
    where M: EventMachine<C>,
          F: FnOnce(M, &mut Scope<C>) -> Response<M>
{
    let mut nmachine = None;
    {
        let ref mut scope = scope(token, &mut handler.context,
            &mut handler.channel, eloop);
        handler.slab.replace_with(token, |m| {
            let res = fun(m, scope);
            nmachine = res.1;
            res.0
        }).ok();  // Spurious events are ok in mio
    }
    while let Some(m) = nmachine.take() {
        handler.add_root(eloop, m);
        let ref mut scope = scope(token, &mut handler.context,
            &mut handler.channel, eloop);
        handler.slab.replace_with(token, |m| {
            let res = m.spawned(scope);
            nmachine = res.1;
            res.0
        }).ok();  // We know that machine is here
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

