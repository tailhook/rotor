//! The mio-based framework for doing I/O in simple and composable way
//!
//! More documentation in [the guide](http://rotor.readthedocs.org)
//!
#![crate_name="rotor"]

extern crate mio as mio_original;
#[macro_use] extern crate quick_error;

mod handler;
mod scope;
mod loop_api;
mod response;
mod compose;
mod macros;
mod machine;
mod notify;

pub use handler::{Handler, NoSlabSpace};
pub use machine::Machine;
pub use scope::Scope;
pub use notify::Notifier;

pub use compose::{Compose2};

// Re-export mio types used in rotor
pub use mio::{EventSet, Evented, PollOpt, EventLoop};
pub use mio::{Timeout, TimerError};
pub use mio_original as mio;


/// The response of a state machine to the (mio) action
///
/// This value is returned by many methods of the `Machine` trait.
// The following struct is not enum
// merely to keep internal structure private
pub struct Response<M, N>(Option<M>, Option<N>);
