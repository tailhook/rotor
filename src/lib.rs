//! The mio-based framework for doing I/O in simple and composable way
//!
//! More documentation in [the guide](http://rotor.readthedocs.org)
//!
#![crate_name="rotor"]

extern crate netbuf;
extern crate time;
extern crate mio;
#[macro_use] extern crate log;
extern crate memchr;

use mio::{Token, Sender};
use std::sync::{Arc, Mutex};

#[macro_use] pub mod async;
pub mod transports;
pub mod handler;
pub mod buffer_util;
mod base;
mod scope;
mod future;
mod loop_api;
//mod timeproto;

pub use base::BaseMachine;
pub use handler::{EventMachine, Handler};
pub use async::Async;
//pub use timeproto::Timer;
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

