use std::any::Any;
use std::error::Error;
use mio::EventSet;

use {Response, Scope};


pub trait Machine<C>: Sized {
    /// Seed is piece of data that is needed to initialize the machine
    ///
    /// It needs Any because it's put into Box<Error> object when state machine
    /// is failed to create. Hopefully this is not huge limitation.
    type Seed: Any+Sized;

    /// Create a machine from some data
    ///
    /// The error should be rare enough so that Box<Error> overhead
    /// is negligible. Most errors here should be resource exhaustion, like
    /// there are no slots in Slab or system limit on epoll watches exceeded.
    fn create(seed: Self::Seed, scope: &mut Scope<C>)
        -> Result<Self, Box<Error>>;

    /// Socket readiness notification
    fn ready(self, events: EventSet, scope: &mut Scope<C>)
        -> Response<Self, Self::Seed>;

    /// Called after spawn event
    ///
    /// This is mostly a continuation event. I.e. when you accept a socket
    /// and return a new state machine from `ready()`. You may wish to accept
    /// another socket right now. This is what `spawned` event is for.
    fn spawned(self, scope: &mut Scope<C>)
        -> Response<Self, Self::Seed>;

    /// Called instead of spawned, if there is no slab space
    ///
    /// For example, in `accept` handler you might want to put the thing
    /// into temporary storage, stop accepting and wait until slot is empty
    /// again.
    ///
    /// Note: it's useless to spawn from here, so we expect Option<Self> here.
    fn spawn_error(self, _scope: &mut Scope<C>, error: Box<Error>)
        -> Option<Self>
    {
        panic!("Error spawning state machine: {}", error);
    }

    /// Timeout happened
    fn timeout(self, scope: &mut Scope<C>) -> Response<Self, Self::Seed>;

    /// Message received
    fn wakeup(self, scope: &mut Scope<C>) -> Response<Self, Self::Seed>;
}
