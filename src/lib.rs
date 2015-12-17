//! The mio-based framework for doing I/O in simple and composable way
//!
//! More documentation in [the guide](http://rotor.readthedocs.org)
//!
#![crate_name="rotor"]

extern crate mio;
extern crate void; // to implement traits

use mio::{Token, Sender};
use std::sync::{Arc, Mutex};

mod handler;
mod scope;
mod future;
mod loop_api;
mod response;
mod compose;
mod macros;
mod machine;
mod creator;

pub use handler::Handler;
pub use machine::Machine;
pub use creator::{Creator, CreationError};
pub use scope::Scope;
pub use loop_api::LoopApi;

pub use compose::{Compose2};

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

// The following struct is not enum
// merely to keep internal structure private
pub struct Response<M, N>(Option<M>, Option<N>);
