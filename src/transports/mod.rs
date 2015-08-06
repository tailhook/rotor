use std::io::{Read, Write};

use mio::Evented;

pub mod greedy_stream;

pub trait StreamSocket: Read + Write + Evented {}

