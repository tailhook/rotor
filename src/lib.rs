//! The mio-based framework for doing I/O in simple and composable way
//!
//! More documentation in [the guide](http://rotor.readthedocs.org)
//!
#![crate_name="rotor"]

extern crate mio;

mod handler;
mod scope;
mod loop_api;
mod response;
mod compose;
mod macros;
mod machine;

pub use handler::{Handler, NoSlabSpace};
pub use machine::Machine;
pub use scope::Scope;
pub use loop_api::LoopApi;

pub use compose::{Compose2};

// The following struct is not enum
// merely to keep internal structure private
pub struct Response<M, N>(Option<M>, Option<N>);
