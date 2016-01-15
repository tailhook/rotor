use std::io;
use std::ops::{Deref, DerefMut};

use mio::{Token, Sender};

use handler::Notify;
use loop_api::LoopApi;
use notify::create_notifier;
use {Notifier};
use {Evented, EventSet, PollOpt, Timeout, TimerError};

/// The structure passed to every action handler
///
/// Scope is used for the following purposes:
///
/// 1. Register/deregister sockets in the event loop
/// 2. Register timeouts
/// 3. Create a special `Notifier` object to wakeup sibling state machines
/// 4. Access to global state of the loop (Context)
///
/// All methods here operate on **enclosed state machine**, which means the
/// state machine that was called with this scope. Or in other words the
/// state machine that actually performs an action.
///
/// The only way to notify another state machine is to create a `notifier()`
/// (the `Notifier` is only able to wakeup this state machine still), transfer
/// it to another state machine (for example putting it into the context)
/// and call `Notifier::wakeup()`.
///
/// The structure derefs to the context (``C``) for convenience
pub struct Scope<'a, C:Sized+'a>{
    token: Token,
    ctx: &'a mut C,
    channel: &'a mut Sender<Notify>,
    loop_api: &'a mut LoopApi,
}

impl<'a, C:Sized+'a> Scope<'a, C> {

    pub fn register(&mut self, io: &Evented, interest: EventSet, opt: PollOpt)
        -> io::Result<()>
    {
        self.loop_api.register(io, self.token, interest, opt)
    }

    pub fn reregister(&mut self, io: &Evented,
        interest: EventSet, opt: PollOpt)
        -> io::Result<()>
    {
        self.loop_api.reregister(io, self.token, interest, opt)
    }

    pub fn deregister(&mut self, io: &Evented) -> io::Result<()>
    {
        self.loop_api.deregister(io)
    }

    pub fn timeout_ms(&mut self, delay: u64) -> Result<Timeout, TimerError>
    {
        self.loop_api.timeout_ms(self.token, delay)
    }

    pub fn clear_timeout(&mut self, token: Timeout) -> bool
    {
        self.loop_api.clear_timeout(token)
    }

    /// Create a `Notifier` that may be used to `wakeup` enclosed state machine
    pub fn notifier(&mut self) -> Notifier {
        create_notifier(self.token, self.channel)
    }

    /// Shutdown the event loop
    pub fn shutdown_loop(&mut self) {
        self.loop_api.shutdown()
    }
}

impl<'a, C> Deref for Scope<'a, C> {
    type Target = C;
    fn deref(&self) -> &C {
        self.ctx
    }
}

impl<'a, C> DerefMut for Scope<'a, C> {
    fn deref_mut(&mut self) -> &mut C {
        self.ctx
    }
}

pub fn scope<'x, C, L:LoopApi>(token: Token, ctx: &'x mut C,
    channel: &'x mut Sender<Notify>, loop_api: &'x mut L)
    -> Scope<'x, C>
{
    Scope {
        token: token,
        ctx: ctx,
        channel: channel,
        loop_api: loop_api,
    }
}
