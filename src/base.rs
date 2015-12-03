use async::Async;
use {Scope};


pub trait BaseMachine<C>: Sized {
    type Value: Sized;
    type State: Sized;

    /// Timeout happened
    fn timeout(self, scope: &mut Scope<C>)
        -> Async<Self, Self::Value, Self::State>
    {
        Async::Ignore(self)
    }

    /// Message received
    fn wakeup(self, scope: &mut Scope<C>)
        -> Async<Self, Self::Value, Self::State>
    {
        Async::Ignore(self)
    }
}
