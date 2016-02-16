use void::Void;

use {Response, Scope, EventSet, SpawnError};


/// A trait that every state machine in the loop must implement
pub trait Machine: Sized {
    /// Context type for the state machine
    ///
    /// This is a container of the global state for the application
    type Context;
    /// Seed is piece of data that is needed to initialize the machine
    ///
    /// It needs Any because it's put into Box<Error> object when state machine
    /// is failed to create. Hopefully this is not huge limitation.
    ///
    /// Note: this is only used to create machines returned by this machine.
    /// So unless this machine processses accepting socket this should
    /// probably be Void.
    type Seed: Sized;

    /// Create a machine from some data
    ///
    /// The error should be rare enough so that Box<Error> overhead
    /// is negligible. Most errors here should be resource exhaustion, like
    /// there are no slots in Slab or system limit on epoll watches exceeded.
    ///
    /// Note: this method is used internally (by event loop) to create a
    /// socket from a Seed returned by this machine. This method should
    /// **not** be used to create machine by external code. Create a
    /// machine-specific `Type::new` method for the purpose.
    ///
    /// Note: we don't support spawning more state machines in create handler
    fn create(seed: Self::Seed, scope: &mut Scope<Self::Context>)
        -> Response<Self, Void>;

    /// Socket readiness notification
    fn ready(self, events: EventSet, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>;

    /// Called after spawn event
    ///
    /// This is mostly a continuation event. I.e. when you accept a socket
    /// and return a new state machine from `ready()`. You may wish to accept
    /// another socket right now. This is what `spawned` event is for.
    fn spawned(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>;

    /// Called instead of spawned, if there is no slab space
    ///
    /// For example, in `accept` handler you might want to put the thing
    /// into temporary storage, stop accepting and wait until slot is empty
    /// again.
    ///
    /// Note: it's useless to spawn from here if the failure was , it almost certainly will fail
    /// again, but may use a timeout
    fn spawn_error(self, _scope: &mut Scope<Self::Context>,
                   error: SpawnError<Self::Seed>)
        -> Response<Self, Self::Seed>
    {
        panic!("Error spawning state machine: {}", error);
    }

    /// Timeout happened
    fn timeout(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>;

    /// Message received
    ///
    /// Note the spurious wakeups are possible, because messages are
    /// asynchronous, and state machine is identified by token.
    /// Tokens are reused quickly.
    ///
    /// So never make this `unreachable!()` or `unimplemented!()`
    fn wakeup(self, scope: &mut Scope<Self::Context>)
        -> Response<Self, Self::Seed>;
}
