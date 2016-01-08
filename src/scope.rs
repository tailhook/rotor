use std::io;
use std::sync::{Arc, Mutex};
use std::ops::{Deref, DerefMut};

use mio::{Token, Sender, Evented, EventSet, PollOpt, Timeout, TimerError};

use {Notify, Future, Port, LoopApi};

/// The object that is passed down to every state machine action handler
///
/// Scope mutably defereferences to the encapsulateed Context. And has a
/// methods to register events to be delivered to the state machine.
///
/// Technically speaking it contains a mio Token, mutable reference to the
/// context, mutable reference to the main loop.
pub struct Scope<'a, C:Sized+'a>{
    token: Token,
    ctx: &'a mut C,
    channel: &'a mut Sender<Notify>,
    loop_api: &'a mut LoopApi,
}

impl<'a, C:Sized+'a> Scope<'a, C> {
    /// Register IO event within the main loop
    ///
    /// Event will always be delivered to the encapsulated state machine.
    ///
    /// While you can register multiple sockets to the same state machine, you
    /// will not be able to differentiate between them. So use on your own
    /// risk. The usual pattern is to use a state machine per socket.
    pub fn register(&mut self, io: &Evented, interest: EventSet, opt: PollOpt)
        -> io::Result<()>
    {
        self.loop_api.register(io, self.token, interest, opt)
    }
    /// Reregister IO event within the main loop
    ///
    /// Considerations are similar to the ones in `Scope::register`
    pub fn reregister(&mut self, io: &Evented,
        interest: EventSet, opt: PollOpt)
        -> io::Result<()>
    {
        self.loop_api.reregister(io, self.token, interest, opt)
    }
    /// Deregister IO event within the main loop
    ///
    /// Note: mio doesn't guarantee that event won't be delivered after the
    /// `deregister` call. It may call `ready` if event is already queued.
    /// (e.g. if you `deregister` from a timeout handler, or even if you
    /// remove it in `ready` on some systems, known case is kqueue). So
    /// be prepared to ignore spurious events in `ready` handler.
    ///
    /// You don't have to deregister events if your socket is oned by the
    /// state machine and will be closed when machine dies. Kernel is able
    /// to do that for you.
    pub fn deregister(&mut self, io: &Evented) -> io::Result<()>
    {
        self.loop_api.deregister(io)
    }

    /// Add timeout to the event loop
    ///
    /// Note that unlike with `Scope::deregister` you really have to remove
    /// the timer or otherwise timers will fill the timer table until they
    /// are expired.
    pub fn timeout_ms(&mut self, delay: u64) -> Result<Timeout, TimerError>
    {
        self.loop_api.timeout_ms(self.token, delay)
    }
    /// Remove the timeout from the main loop
    ///
    /// Note that unlike with `Scope::deregister` you really have to remove
    /// the timer or otherwise timers will fill the timer table until they
    /// are expired.
    pub fn clear_timeout(&mut self, token: Timeout) -> bool
    {
        self.loop_api.clear_timeout(token)
    }
    /// Turn the scope reference into scope with different context
    ///
    /// This is useful when you compose multiple applications which require
    /// same traits on the context to return different values. This should
    /// be used in the state machine that does composition. It can map the
    /// scope for each child state machine to the different context instance.
    pub fn map_context<'x:'y, 'y, N, F>(&'x mut self, fun: F) -> Scope<'y, N>
        where N: Sized, F: FnOnce(&'x mut C) -> &'y mut N
    {
        Scope {
            token: self.token,
            ctx: fun(self.ctx),
            channel: self.channel,
            loop_api: self.loop_api,
        }
    }
}

fn pair<T:Sized>(token: Token, channel: &Sender<Notify>)
    -> (Port<T>, Future<T>)
{
    let arc = Arc::new(Mutex::new(None::<T>));
    let port = Port {
        token: token,
        contents: Arc::downgrade(&arc),
        channel: channel.clone(),
    };
    let future = Future {
        contents: arc,
    };
    return (port, future);
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

impl<'a, C> Scope<'a, C> {
    pub fn create_future<T:Sized>(&mut self) -> (Port<T>, Future<T>) {
        pair(self.token, self.channel)
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

#[cfg(test)]
mod test {
    use std::error::Error;
    use super::{scope, Scope};
    use super::super::{Response, Handler, Machine};
    use mio;

    struct Stub;

    impl<C:Sized> Machine<C> for Stub {
        type Seed = ();
        fn create(_seed: Self::Seed, _scope: &mut Scope<C>)
            -> Result<Self, Box<Error>>
        { unreachable!() }
        fn ready(self, _events: mio::EventSet, _scope: &mut Scope<C>)
            -> Response<Self, Self::Seed>
        { unreachable!() }
        fn spawned(self, _scope: &mut Scope<C>)
            -> Response<Self, Self::Seed>
        { unreachable!() }
        fn spawn_error(self, _scope: &mut Scope<C>, _error: Box<Error>)
            -> Option<Self>
        { unreachable!() }
        fn timeout(self, _scope: &mut Scope<C>) -> Response<Self, Self::Seed>
        { unreachable!() }
        fn wakeup(self, _scope: &mut Scope<C>) -> Response<Self, Self::Seed>
        { unreachable!() }
    }

    #[test]
    fn map_scope() {
        let mut context = (1u32, 2u64);
        let mut eloop: mio::EventLoop<Handler<(u32, u64), Stub>>;
        eloop = mio::EventLoop::new().unwrap();
        let mut chan = eloop.channel();
        let mut scope = scope(mio::Token(1),
            &mut context, &mut chan, &mut eloop);
        assert_eq!(*scope.map_context(|x| &mut x.0), 1u32);
        assert_eq!(*scope.map_context(|x| &mut x.1), 2u64);
    }
}
