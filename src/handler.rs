use std::io::Error;
use std::usize;

use mio::{self, EventLoop, Token, EventSet, Evented, PollOpt};
use mio::util::Slab;
use mio::{Sender, Timeout, TimerError};

use {Scope, BaseMachine};


#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Abort {
    NoSlabSpace,
    RegisterFailed,
    MachineAddError,
}

pub enum Notify<T> {
    NewMachine(T),
}

struct RootScope<'a, H: mio::Handler>
    where H::Timeout: 'a, H::Message: 'a, H:'a
{
    channel: &'a Sender<H::Message>,
    eloop: &'a mut EventLoop<H>,
    token: Token,
}

pub struct Handler<Ctx, M: Send> {
    slab: Slab<M>,
    context: Ctx,
    channel: Sender<Notify<M>>,
}

pub trait EventMachine<C>: BaseMachine + Send + Sized {
    /// Socket readiness notification
    fn ready<S>(self, events: EventSet, context: &mut C, scope: &mut S)
        -> Option<Self>
        where S: Scope<Self>;

    /// Gives socket a chance to register in event loop
    fn register<S>(&mut self, scope: &mut S)
        -> Result<(), Error>
        where S: Scope<Self>;

    /// Abnormal termination of event machine
    fn abort<S>(self, reason: Abort, _context: &mut C, _scope: &mut S)
        where S: Scope<Self>
    {
        // TODO(tailhook) use Display instead of Debug
        error!("Connection aborted: {:?}", reason);
    }
}

impl<C, M:Send> Handler<C, M>
    where M: EventMachine<C>
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

impl<'a, C, M> Scope<M> for RootScope<'a, Handler<C, M>>
    where M: 'a, M: EventMachine<C>,
          M::Timeout: 'a,
{
    fn async_add_machine(&mut self, m: M) -> Result<(), M> {
        use mio::NotifyError::*;
        match self.channel.send(Notify::NewMachine(m)) {
            Ok(()) => Ok(()),
            Err(Io(e)) => {
                // We would probably do something better here, but mio doesn't
                // give us a message. But anyway it's probably never happen
                panic!("Io error when sending notify: {}", e);
            }
            Err(Full(Notify::NewMachine(m))) => Err(m),
            Err(Closed(_)) => {
                // It should never happen because we usually send from the
                // inside of a main loop
                panic!("Sending to closed channel. Main loop is already shut \
                    down");
            }
        }
    }
    fn add_timeout_ms(&mut self, delay: u64, t: M::Timeout)
        -> Result<Timeout, TimerError>
    {
        self.eloop.timeout_ms(t, delay)
    }
    fn clear_timeout(&mut self, timeout: Timeout) -> bool {
        self.eloop.clear_timeout(timeout)
    }
    fn register<E: ?Sized>(&mut self, io: &E, interest: EventSet, opt: PollOpt)
        -> Result<(), Error>
        where E: Evented
    {
        self.eloop.register_opt(io, self.token, interest, opt)
    }
}

impl<'a, M, Ctx> mio::Handler for Handler<Ctx, M>
    where M: EventMachine<Ctx>
{
    type Message = Notify<M>;
    type Timeout = M::Timeout;
    fn ready<'x>(&mut self, eloop: &'x mut EventLoop<Self>,
        token: Token, events: EventSet)
    {
        let ref mut ctx = self.context;
        let ref mut scope = RootScope {
            eloop: eloop,
            channel: &self.channel,
            token: token,
        };
        self.slab.replace_with(token, |fsm| {
            fsm.ready(events, ctx, scope)
        }).ok();  // Spurious events are ok in mio
    }

    fn notify(&mut self, eloop: &mut EventLoop<Self>, msg: Self::Message) {
        use self::Notify::*;
        let ref mut ctx = self.context;
        match msg {
            NewMachine(fsm) => {
                // This is so complex because of limitations of Slab
                match self.slab.insert(fsm) {
                    Ok(tok) => {
                        let ref mut scope = RootScope {
                            eloop: eloop,
                            channel: &self.channel,
                            token: tok,
                        };
                        self.slab.replace_with(tok, |mut fsm| {
                            match fsm.register(scope) {
                                Ok(()) => Some(fsm),
                                Err(_) => {
                                    fsm.abort(Abort::RegisterFailed,
                                        ctx, scope);
                                    None
                                }
                            }
                        }).unwrap();
                    }
                    Err(fsm) => {
                        // TODO(tailhook) it should be global scope instead
                        // of FSM-bound scope
                        let ref mut scope = RootScope {
                            eloop: eloop,
                            channel: &self.channel,
                            token: Token(usize::MAX),
                        };
                        fsm.abort(Abort::NoSlabSpace, ctx, scope);
                    }
                }
            }
        }
    }
}

