use std::error::Error;

use mio::{self, EventLoop, Token, EventSet, Sender};
use mio::util::Slab;

use scope::scope;
use {SpawnError, Scope, Response, Machine};
use SpawnError::{NoSlabSpace, UserError};


pub enum Timeo {
    Fsm(Token),
}

pub enum Notify {
    Fsm(Token),
}


/// Standard mio loop handler
///
///
/// # Examples
///
/// ```ignore
/// extern crate mio;
/// extern crate rotor;
///
/// let mut event_loop = mio::EventLoop::new().unwrap();
/// let mut handler = rotor::Handler::new(Context, &mut event_loop);
/// let conn = handler.add_machine_with(&mut event_loop, |scope| {
///     Ok(StateMachineConstuctor(..))
/// });
/// assert!(conn.is_ok());
/// event_loop.run(&mut handler).unwrap();
/// ```
pub struct Handler<Ctx, M>
    where M: Machine<Context=Ctx>
{
    slab: Slab<M>,
    context: Ctx,
    channel: Sender<Notify>,
}


impl<C, M> Handler<C, M>
    where M: Machine<Context=C>,
{
    pub fn new_with_capacity(context: C, eloop: &mut EventLoop<Handler<C, M>>,
        capacity: usize)
        -> Handler<C, M>
    {
        Handler {
            slab: Slab::new(capacity),
            context: context,
            channel: eloop.channel(),
        }
    }
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

pub fn create_handler<C, M>(slab: Slab<M>, context: C, channel: Sender<Notify>)
    -> Handler<C, M>
    where M: Machine<Context=C>
{
    Handler {
        slab: slab,
        context: context,
        channel: channel,
    }
}

impl<C, M> Handler<C, M>
    where M: Machine<Context=C>
{
    pub fn add_machine_with<F>(&mut self,
        eloop: &mut EventLoop<Self>, fun: F) -> Result<(), SpawnError<()>>
        where F: FnOnce(&mut Scope<C>) -> Result<M, Box<Error>>
    {
        let ref mut ctx = self.context;
        let ref mut chan = self.channel;
        let res = self.slab.insert_with(|token| {
            let ref mut scope = scope(token, ctx, chan, eloop);
            match fun(scope) {
                Ok(x) => x,
                Err(_) => {
                // TODO(tailhook) when Slab::insert_with_opt() lands, fix it
                    panic!("Unimplemented: Slab::insert_with_opt");
                }
            }
        });
        if res.is_some() {
            Ok(())
        } else {
            Err(NoSlabSpace(()))
        }
    }

}

fn machine_loop<C, M, F>(handler: &mut Handler<C, M>,
    eloop: &mut EventLoop<Handler<C, M>>, token: Token, fun: F)
    where M: Machine<Context=C>,
          F: FnOnce(M, &mut Scope<C>) -> Response<M, M::Seed>
{
    let mut creator = None;
    {
        let ref mut scope = scope(token, &mut handler.context,
            &mut handler.channel, eloop);
        handler.slab.replace_with(token, |m| {
            let res = fun(m, scope);
            creator = res.1;
            res.0
        }).ok();  // Spurious events are ok in mio
    }
    while let Some(new) = creator.take() {
        let mut new = Some(new);
        let res = handler.add_machine_with(eloop, |scope| {
            M::create(new.take().unwrap(), scope)
        });
        if let Err(err) = res {
            let err = if let Some(new) = new.take() {
                NoSlabSpace(new)
            } else if let UserError(e) = err {
                UserError(e)
            } else {
                unreachable!();
            };
            let ref mut scope = scope(token, &mut handler.context,
                &mut handler.channel, eloop);
            handler.slab.replace_with(token, |m| {
                m.spawn_error(scope, err)
            }).ok();
            break;
        } else {
            let ref mut scope = scope(token, &mut handler.context,
                &mut handler.channel, eloop);
            handler.slab.replace_with(token, |m| {
                let res = m.spawned(scope);
                creator = res.1;
                res.0
            }).ok();
        }
    }
}

impl<Ctx, M> mio::Handler for Handler<Ctx, M>
    where M: Machine<Context=Ctx>
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
