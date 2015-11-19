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

#[macro_use] pub mod async;
pub mod transports;
pub mod handler;
pub mod buffer_util;
mod scope;
mod future;
mod timeproto;

pub use handler::{EventMachine, Handler};
pub use async::Async;
pub use scope::Scope;
pub use future::{Future, Port};
pub use timeproto::Timer;
