//! The mio-based framework for doing I/O in simple and composable way
//!
//! More documentation in [the guide](http://rotor.readthedocs.org)
//!
#![crate_name="rotor"]

extern crate void as void_original;
extern crate mio as mio_original;
#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;

mod handler;
mod scope;
mod loop_api;
mod response;
mod compose;
mod macros;
mod machine;
mod notify;
mod config;
mod creator;
mod error;
mod loop_time;

pub use machine::Machine;
pub use scope::{Scope, EarlyScope, GenericScope};
pub use scope::{scope as _scope, early_scope as _early_scope};
pub use notify::{Notifier, WakeupError};
pub use config::Config;
pub use creator::{LoopCreator as Loop, LoopInstance};
pub use error::SpawnError;
pub use loop_time::Time;
pub use handler::{Timeo as _Timeo, Notify as _Notify};
pub use loop_api::{LoopApi as _LoopApi};

pub use compose::{Compose2};

// Re-export mio types used in rotor
pub use mio::{EventSet, Evented, PollOpt};
pub use mio::{Timeout, TimerError};
pub use mio_original as mio;
// Re-export void too
pub use void::{Void};
pub use void_original as void;


/// The response of a state machine to the (mio) action
///
/// This value is returned by many methods of the `Machine` trait.
pub struct Response<M, N>(response::ResponseImpl<M, N>);
