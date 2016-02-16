use std::io;
use std::ops::{Deref, DerefMut};

use mio::{Token, Sender};

use handler::Notify;
use loop_api::LoopApi;
use notify::create_notifier;
use {Notifier, Time};
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
    time: Time,
}

/// This is a structure that works similarly to Scope, but doesn't
/// have a context
///
/// The primary (and probably the only) use case for the `EarlyScope` is to
/// allow to create a state machine before context has been intialized. This
/// is useful if you want to put a `Notifier` of the FSM to a context itself.
pub struct EarlyScope<'a> {
    token: Token,
    channel: &'a mut Sender<Notify>,
    loop_api: &'a mut LoopApi,
}

/// A common part of `Scope` and `EarlyScope`
///
/// For most cases `Scope` scope should be used directly. The trait is here
/// so you can create a constructor for state machine that is generic over
/// type of scope used.
pub trait GenericScope {
    fn register(&mut self, io: &Evented, interest: EventSet, opt: PollOpt)
        -> io::Result<()>;
    fn reregister(&mut self, io: &Evented,
        interest: EventSet, opt: PollOpt)
        -> io::Result<()>;
    fn deregister(&mut self, io: &Evented) -> io::Result<()>;
    fn timeout_ms(&mut self, delay: u64) -> Result<Timeout, TimerError>;
    fn clear_timeout(&mut self, token: Timeout) -> bool;
    fn notifier(&mut self) -> Notifier;
    /// Time of the current loop iteration
    ///
    /// This is a time that needs to be used for timeouts. It's cheap to use
    fn now(&self) -> Time;
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

    /// Time of the current loop iteration
    ///
    /// This is a time that needs to be used for timeouts. It's cheap to use
    pub fn now(&self) -> Time {
        self.time
    }
}

impl<'a, C:Sized+'a> GenericScope for Scope<'a, C> {

    fn register(&mut self, io: &Evented, interest: EventSet, opt: PollOpt)
        -> io::Result<()>
    {
        self.register(io, interest, opt)
    }

    fn reregister(&mut self, io: &Evented,
        interest: EventSet, opt: PollOpt)
        -> io::Result<()>
    {
        self.reregister(io, interest, opt)
    }

    fn deregister(&mut self, io: &Evented) -> io::Result<()>
    {
        self.deregister(io)
    }

    fn timeout_ms(&mut self, delay: u64) -> Result<Timeout, TimerError>
    {
        self.timeout_ms(delay)
    }

    fn clear_timeout(&mut self, token: Timeout) -> bool
    {
        self.clear_timeout(token)
    }

    /// Create a `Notifier` that may be used to `wakeup` enclosed state machine
    fn notifier(&mut self) -> Notifier {
        self.notifier()
    }

    /// Time of the current loop iteration
    ///
    /// This is a time that needs to be used for timeouts. It's cheap to use
    fn now(&self) -> Time {
        self.time
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

impl<'a> EarlyScope<'a> {

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

    /// Time of the current loop iteration
    ///
    /// This is a time that needs to be used for timeouts. It's cheap to use
    pub fn now(&self) -> Time {
        // Early scope is only used when creating a context. It's definitely
        // at the start of the things. But we may review this in future.
        Time::zero()
    }
}

impl<'a> GenericScope for EarlyScope<'a> {

    fn register(&mut self, io: &Evented, interest: EventSet, opt: PollOpt)
        -> io::Result<()>
    {
        self.register(io, interest, opt)
    }

    fn reregister(&mut self, io: &Evented,
        interest: EventSet, opt: PollOpt)
        -> io::Result<()>
    {
        self.reregister(io, interest, opt)
    }

    fn deregister(&mut self, io: &Evented) -> io::Result<()>
    {
        self.deregister(io)
    }

    fn timeout_ms(&mut self, delay: u64) -> Result<Timeout, TimerError>
    {
        self.timeout_ms(delay)
    }

    fn clear_timeout(&mut self, token: Timeout) -> bool
    {
        self.clear_timeout(token)
    }

    /// Create a `Notifier` that may be used to `wakeup` enclosed state machine
    fn notifier(&mut self) -> Notifier {
        self.notifier()
    }
    /// Time of the current loop iteration
    ///
    /// This is a time that needs to be used for timeouts. It's cheap to use
    fn now(&self) -> Time {
        // Early scope is only used when creating a context. It's definitely
        // at the start of the things. But we may review this in future.
        Time::zero()
    }
}

pub fn scope<'x, C, L:LoopApi>(time: Time, token: Token, ctx: &'x mut C,
    channel: &'x mut Sender<Notify>, loop_api: &'x mut L)
    -> Scope<'x, C>
{
    Scope {
        token: token,
        ctx: ctx,
        channel: channel,
        loop_api: loop_api,
        time: time,
    }
}

pub fn early_scope<'x, L:LoopApi>(token: Token,
    channel: &'x mut Sender<Notify>, loop_api: &'x mut L)
    -> EarlyScope<'x>
{
    EarlyScope {
        token: token,
        channel: channel,
        loop_api: loop_api,
    }
}
