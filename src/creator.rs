use void::{Void, unreachable};

use {Scope, Machine};

pub trait Creator<C>: Sized {
    type Machine: Machine<C>;
    type Error: Sized;
    fn create(self, scope: &mut Scope<C>)
        -> Result<Self::Machine, Self::Error>;
}

pub enum CreationError<C:Sized, E:Sized> {
    /// Returned when there is no slab space available to insert new machine
    OutOfResources(C),
    /// The `Creator::create` returned a failure
    CreatorFailure(E),
}

impl<T> Creator<T> for Void {
    type Machine = Void;
    type Error = Void;
    fn create(self, scope: &mut Scope<T>)
        -> Result<Self::Machine, Self::Error> {
        unreachable(self);
    }
}
