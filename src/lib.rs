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

pub use handler::{EventMachine, Handler};
pub use async::Async;
