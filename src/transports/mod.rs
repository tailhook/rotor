use std::io::{Read, Write};

use mio::Evented;

pub mod greedy_stream;
pub mod accept;

pub trait StreamSocket: Read + Write + Evented {}

