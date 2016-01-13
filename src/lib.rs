//! The mio-based framework for doing I/O in simple and composable way
//!
//! More documentation in [the guide](http://rotor.readthedocs.org)
//!
#![crate_name="rotor"]

extern crate mio;

use mio::{Token, Sender};
use std::sync::{Arc, Weak, Mutex};

mod handler;
mod scope;
mod future;
mod loop_api;
mod response;
mod compose;
mod macros;
mod machine;
mod monitor;
#[cfg(test)] mod test_util;

pub use handler::{Handler, NoSlabSpace};
pub use machine::Machine;
pub use scope::Scope;
pub use loop_api::LoopApi;
pub use monitor::{Guard as MonitorGuard, Monitor};
pub use monitor::{Peer1Monitor, Peer2Monitor, Peer1Socket, Peer2Socket};
pub use monitor::{create_pair};

pub use compose::{Compose2};

pub enum Notify {
    Fsm(Token),
}

pub struct Port<T: Sized> {
    token: Token,
    contents: Weak<Mutex<Option<T>>>,
    channel: Sender<Notify>,
}

pub struct Future<T: Sized> {
    contents: Arc<Mutex<Option<T>>>,
}

// The following struct is not enum
// merely to keep internal structure private
pub struct Response<M, N>(Option<M>, Option<N>);
