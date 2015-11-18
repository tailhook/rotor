use std::sync::{Arc, Mutex};

use mio::{Token, Sender};

use handler::Notify;

pub struct Port<T: Sized> {
    token: Token,
    contents: Arc<Mutex<Option<T>>>,
    channel: Sender<Notify>,
}

pub struct Future<T: Sized> {
    contents: Arc<Mutex<Option<T>>>,
}

pub fn pair<T:Sized>(token: Token, channel: &Sender<Notify>)
    -> (Port<T>, Future<T>)
{
    let arc = Arc::new(Mutex::new(None::<T>));
    let port = Port {
        token: token,
        contents: arc.clone(),
        channel: channel.clone(),
    };
    let future = Future {
        contents: arc,
    };
    return (port, future);
}

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
