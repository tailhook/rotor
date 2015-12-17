use mio::EventSet;

use {Response, Scope, Creator, CreationError};


pub trait Machine<C>: Sized {
    type Creator: Creator<C, Machine=Self>;

    /// Socket readiness notification
    fn ready(self, events: EventSet, scope: &mut Scope<C>)
        -> Response<Self, Self::Creator>;

    /// Called after spawn event
    ///
    /// This is mostly a continuation event. I.e. when you accept a socket
    /// and return a new state machine from `ready()`. You may wish to accept
    /// another socket right now. This is what `spawned` event is for.
    fn spawned(self, scope: &mut Scope<C>)
        -> Response<Self, Self::Creator>;

    /// Called instead of spawned, if there is no slab space
    ///
    /// For example, in `accept` handler you might want to put the thing
    /// into temporary storage, stop accepting and wait until slot is empty
    /// again.
    ///
    /// Note: it's useless to spawn from here, so we expect Option<Self> here.
    fn spawn_error(self, _scope: &mut Scope<C>,
        _error: CreationError<Self::Creator, <Self::Creator as Creator<C>>::Error>)
        -> Option<Self>
    {
        panic!("Out of slab space in rotor/mio main loop");
    }

    /// Timeout happened
    fn timeout(self, scope: &mut Scope<C>) -> Response<Self, Self::Creator>;

    /// Message received
    fn wakeup(self, scope: &mut Scope<C>) -> Response<Self, Self::Creator>;
}
