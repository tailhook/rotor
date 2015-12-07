use std::io::{Read, Write};

use mio::Evented;

//pub mod stream;
pub mod accept;

pub trait StreamSocket: Read + Write + Evented {}
impl<T> StreamSocket for T where T: Read, T: Write, T: Evented {}

