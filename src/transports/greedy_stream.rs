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
use std::marker::PhantomData;
use std::io::ErrorKind::{WouldBlock, Interrupted};

use mio::{EventSet, Token, EventLoop, PollOpt, Evented, Handler};
use netbuf::Buf;

use super::StreamSocket as Socket;
use super::super::handler::EventMachine;
use super::accept::Init;



impl<T> Socket for T where T: Read, T: Write, T: Evented {}


struct Inner<S: Socket+Send> {
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

pub struct Stream<S: Socket+Send, P: Protocol<C>, C>(
    Inner<S>, P, PhantomData<*const C>);

unsafe impl<S: Socket+Send, P: Protocol<C>+Send, C> Send for Stream<S, P, C> {}

/// This trait you should implement to handle the protocol. Only data_received
/// handler is required, everything else may be left as is.
pub trait Protocol<C>: Send + Sized {
    /// Returns new state machine in a state for new accepted connection
    // TODO(tailhook) should socket address be passed here?
    fn accepted(ctx: &mut C) -> Self;
    /// Some chunk of data has been received and placed into the buffer
    ///
    /// It's edge-triggered so be sure to read everything useful. But you
    /// can leave half-received packets in the buffer
    fn data_received(self, transport: &mut Transport, ctx: &mut C)
        -> Option<Self>;

    /// Eof received. State machine will shutdown unconditionally
    fn eof_received(self, _ctx: &mut C) {}

    /// Fatal error on connection happened, you may process error somehow, but
    /// statemachine will be destroyed anyway (note you receive self)
    ///
    /// Default action is to log error on the info level
    fn error_happened(self, e: Error, _ctx: &mut C) {
        info!("Error when handling connection: {}", e);
    }
}

impl<S: Socket+Send, P:Protocol<C>, C> Init<S, C> for Stream<S, P, C>  {
    fn accept(sock: S, ctx: &mut C) -> Self {
        Stream(Inner {
            sock: sock,
            inbuf: Buf::new(),
            outbuf: Buf::new(),
            readable: false,
            writable: true,   // Accepted socket is immediately writable
        }, Protocol::accepted(ctx), PhantomData)
    }
}

impl<S: Socket+Send, P:Protocol<C>, C> EventMachine<C> for Stream<S, P, C> {
    fn ready(self, evset: EventSet, context: &mut C)
        -> Option<Stream<S, P, C>>
    {
        let Stream(mut stream, mut fsm, _) = self;
        if evset.is_writable() && stream.outbuf.len() > 0 {
            stream.writable = true;
            while stream.outbuf.len() > 0 {
                match stream.outbuf.write_to(&mut stream.sock) {
                    Ok(0) => { // Connection closed
                        fsm.eof_received(context);
                        return None;
                    }
                    Ok(_) => {}  // May notify application
                    Err(ref e) if e.kind() == WouldBlock => {
                        stream.writable = false;
                        break;
                    }
                    Err(ref e) if e.kind() == Interrupted =>  { continue; }
                    Err(e) => {
                        fsm.error_happened(e, context);
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
                        fsm.eof_received(context);
                        return None;
                    }
                    Ok(_) => {
                        fsm = match fsm.data_received(&mut Transport {
                            inbuf: &mut stream.inbuf,
                            outbuf: &mut stream.outbuf,
                        }, context) {
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
                        fsm.error_happened(e, context);
                        return None;
                    }
                }
            }
        }
        if stream.writable && stream.outbuf.len() > 0 {
            while stream.outbuf.len() > 0 {
                match stream.outbuf.write_to(&mut stream.sock) {
                    Ok(0) => { // Connection closed
                        fsm.eof_received(context);
                        return None;
                    }
                    Ok(_) => {}  // May notify application
                    Err(ref e) if e.kind() == WouldBlock => {
                        stream.writable = false;
                        break;
                    }
                    Err(ref e) if e.kind() == Interrupted =>  { continue; }
                    Err(e) => {
                        fsm.error_happened(e, context);
                        return None;
                    }
                }
            }
        }
        Some(Stream(stream, fsm, PhantomData))
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
