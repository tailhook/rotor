use std::io;
use std::default::Default;

use mio::deprecated::{EventLoop, EventLoopBuilder};

use handler::Handler;
use {Machine, Slab};


/// Event loop configuration
///
/// The structure currently embeds mio configuration too
#[derive(Debug, Clone)]
pub struct Config {
    mio: EventLoopBuilder,
    slab_capacity: usize,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            mio: Default::default(),
            slab_capacity: 4096,
        }
    }
}

impl Config {
    /// Create new configuration with default options
    pub fn new() -> Config {
        Config {
            mio: EventLoopBuilder::new(),
            slab_capacity: 4096,
        }
    }
    /// A mutable reference for ``mio::EventLoopBuilder``
    pub fn mio(&mut self) -> &mut EventLoopBuilder {
        &mut self.mio
    }
    /// A capacity of state machine slab
    ///
    /// This limits the number of state machines that application is able
    /// to create. Consequently this limits the number of connections that
    /// server is able to establish.
    pub fn slab_capacity(&mut self, capacity: usize) {
        self.slab_capacity = capacity;
    }
}


// The functions are probably don't belong here, but they accept otherwise
// private parts of the config structure

pub fn create_slab<M:Sized>(cfg: &Config) -> Slab<M> {
    Slab::with_capacity(cfg.slab_capacity)
}

pub fn create_loop<M: Machine>(cfg: &Config)
    -> Result<EventLoop<Handler<M>>, io::Error>
{
    cfg.mio.clone().build()
}
