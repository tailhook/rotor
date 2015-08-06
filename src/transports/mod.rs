use std::io::{Read, Write};

pub mod greedy_stream;

pub trait StreamSocket: Read + Write {}

