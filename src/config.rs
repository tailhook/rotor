use std::io;
use std::default::Default;

use mio::util::Slab;
use mio::{EventLoop, EventLoopConfig};

use handler::Handler;
use {Machine};


/// Event loop configuration
///
/// The structure currently embeds mio configuration too
#[derive(Debug, Clone)]
pub struct Config {
    mio: EventLoopConfig,
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
            mio: EventLoopConfig::new(),
            slab_capacity: 4096,
        }
    }
    /// A mutable reference for ``mio::EventLoopConfig``
    pub fn mio(&mut self) -> &mut EventLoopConfig {
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
    Slab::new(cfg.slab_capacity)
}

pub fn create_loop<M: Machine>(cfg: &Config)
    -> Result<EventLoop<Handler<M>>, io::Error>
{
    EventLoop::configured(cfg.mio.clone())
}
