use std::sync::{Arc, Mutex};

use mio::{Token, Sender};

use {Notify, Port, Future};

impl<T:Sized> Port<T> {
    /// Set the value of the future
    ///
    /// # Panics
    ///
    /// Panics when message to the target main loop can't be sent
    pub fn set(self, value: T) {
        *self.contents.lock()
            .expect("Lock of the future is poisoned") = Some(value);
        self.channel.send(Notify::Fsm(self.token))
            .expect("Target channel for the future is full");
    }
}

impl<T:Sized> Future<T> {
    /// Get the value consuming the future
    ///
    /// # Panics
    ///
    /// Panics when no value is set yet
    pub fn get(self) -> T {
        self.contents.lock().expect("Lock of the future is poisoned")
        .take().expect("Future is not resolved yet")
    }
    pub fn done(&self) -> bool {
        self.contents.lock()
            .expect("Lock of the future is poisoned").is_some()
    }
}
