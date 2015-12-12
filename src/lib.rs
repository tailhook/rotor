//! The mio-based framework for doing I/O in simple and composable way
//!
//! More documentation in [the guide](http://rotor.readthedocs.org)
//!
#![crate_name="rotor"]

extern crate mio;

use mio::{Token, Sender};
use std::sync::{Arc, Mutex};

mod handler;
mod scope;
mod future;
mod loop_api;

pub use handler::{EventMachine, Handler};
pub use scope::Scope;
pub use loop_api::LoopApi;

pub enum Notify {
    Fsm(Token),
}

pub struct Port<T: Sized> {
    token: Token,
    contents: Arc<Mutex<Option<T>>>,
    channel: Sender<Notify>,
}

pub struct Future<T: Sized> {
    contents: Arc<Mutex<Option<T>>>,
}

pub struct Response<M: Sized>(Option<M>, Option<M>);

impl<M: Sized> Response<M> {
    pub fn ok(machine: M) -> Response<M> {
        Response(Some(machine), None)
    }
    pub fn spawn(machine: M, result: M) -> Response<M> {
        Response(Some(machine), Some(result))
    }
    pub fn done() -> Response<M> {
        Response::<M>(None, None)
    }
}
