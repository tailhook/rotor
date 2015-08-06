//! The simplest to use transport a `gready_stream`
//!
//! The transport agressively reads everything from input socket putting it
//! into the buffer. Similarly everything put into output buffer is just sent
//! back to the user.
//!
//! It's assumed that Protocol is able to keep up with the input rate. But if
//! it's not always the case you can always see input buffer size and drop
//! a connection. Currently you can't throttle neither reading end nor writer
//! (i.e. don't put everything to the output buffer at once)
//!
//! This is tradeoff to have super simple protocol and semantics. More
//! elaborate protocols will be implemented in the future.
//!
use std::io::{Read, Write, Error};
use std::io::ErrorKind::{WouldBlock, Interrupted};

use mio::{EventSet, Token, EventLoop, PollOpt, Evented, Handler};
use netbuf::Buf;

use super::StreamSocket as Socket;
use super::super::handler::EventMachine;



impl<T> Socket for T where T: Read, T: Write, T: Evented {}


struct Stream<S: Socket+Send> {
    sock: S,
    inbuf: Buf,
    outbuf: Buf,
    writable: bool,
    readable: bool,
}

pub struct Transport<'a> {
    inbuf: &'a mut Buf,
    outbuf: &'a mut Buf,
}

pub struct StreamMachine<S: Socket+Send, C: Protocol>(Stream<S>, C);

/// This trait you should implement to handle the protocol. Only data_received
/// handler is required, everything else may be left as is.
pub trait Protocol: Send + Sized {
    /// Some chunk of data has been received and placed into the buffer
    ///
    /// It's edge-triggered so be sure to read everything useful. But you
    /// can leave half-received packets in the buffer
    fn data_received(self, transport: &mut Transport) -> Option<Self>;

    /// Eof received. State machine will shutdown unconditionally
    fn eof_received(self) {}

    /// Fatal error on connection happened, you may process error somehow, but
    /// statemachine will be destroyed anyway (note you receive self)
    ///
    /// Default action is to log error on the info level
    fn error_happened(self, e: Error) {
        info!("Error when handling connection: {}", e);
    }
}

impl<S: Socket+Send, C: Protocol> StreamMachine<S, C> {
    pub fn new(sock: S, proto: C) -> StreamMachine<S, C> {
        StreamMachine(Stream {
            sock: sock,
            inbuf: Buf::new(),
            outbuf: Buf::new(),
            readable: false,
            writable: false,
        }, proto)
    }
}

impl<S: Socket+Send, C:Protocol> EventMachine for StreamMachine<S, C> {
    fn ready(self, evset: EventSet) -> Option<StreamMachine<S, C>> {
        let StreamMachine(mut stream, mut fsm) = self;
        if evset.is_writable() && stream.outbuf.len() > 0 {
            stream.writable = true;
            while stream.outbuf.len() > 0 {
                match stream.outbuf.write_to(&mut stream.sock) {
                    Ok(0) => { // Connection closed
                        fsm.eof_received();
                        return None;
                    }
                    Ok(_) => {}  // May notify application
                    Err(ref e) if e.kind() == WouldBlock => {
                        stream.writable = false;
                        break;
                    }
                    Err(ref e) if e.kind() == Interrupted =>  { continue; }
                    Err(e) => {
                        fsm.error_happened(e);
                        return None;
                    }
                }
            }
        }
        if evset.is_readable() {
            stream.readable = true;
            loop {
                match stream.inbuf.read_from(&mut stream.sock) {
                    Ok(0) => { // Connection closed
                        fsm.eof_received();
                        return None;
                    }
                    Ok(_) => {
                        fsm = match fsm.data_received(&mut Transport {
                            inbuf: &mut stream.inbuf,
                            outbuf: &mut stream.outbuf,
                        }) {
                            Some(fsm) => fsm,
                            None => return None,
                        };
                    }
                    Err(ref e) if e.kind() == WouldBlock => {
                        stream.readable = false;
                        break;
                    }
                    Err(ref e) if e.kind() == Interrupted =>  { continue; }
                    Err(e) => {
                        fsm.error_happened(e);
                        return None;
                    }
                }
            }
        }
        if stream.writable && stream.outbuf.len() > 0 {
            while stream.outbuf.len() > 0 {
                match stream.outbuf.write_to(&mut stream.sock) {
                    Ok(0) => { // Connection closed
                        fsm.eof_received();
                        return None;
                    }
                    Ok(_) => {}  // May notify application
                    Err(ref e) if e.kind() == WouldBlock => {
                        stream.writable = false;
                        break;
                    }
                    Err(ref e) if e.kind() == Interrupted =>  { continue; }
                    Err(e) => {
                        fsm.error_happened(e);
                        return None;
                    }
                }
            }
        }
        Some(StreamMachine(stream, fsm))
    }

    fn register<H:Handler>(&mut self, tok: Token, eloop: &mut EventLoop<H>)
        -> Result<(), Error>
    {
        eloop.register_opt(&self.0.sock, tok, EventSet::all(), PollOpt::edge())
    }
}

impl<'a> Transport<'a> {
    pub fn input<'x>(&'x mut self) -> &'x mut Buf {
        self.inbuf
    }
    pub fn output<'x>(&'x mut self) -> &'x mut Buf {
        self.outbuf
    }
}
