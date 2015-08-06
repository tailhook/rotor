use std::io::Error;

use mio::{self, EventLoop, Token, EventSet};
use mio::util::Slab;


pub enum Notify<T:EventMachine> {
    NewMachine(T),
}

pub trait EventMachine: Send + Sized {

    /// Socket readiness notification
    fn ready(self, events: EventSet) -> Option<Self>;

    /// Gives socket a chance to register in event loop
    ///
    /// Keep in mind that there are no reregister or any other kind of modify
    /// operation. So it's mostly useful for registering all events in
    /// edge-triggered mode
    fn register<H: mio::Handler>(&mut self,
        tok: Token, eloop: &mut EventLoop<H>)
        -> Result<(), Error>;

    /// Abnormal termination of event machine
    ///
    /// Currently happens when no slab space available or when register
    /// fails (which presumably means can't add socket to epoll)
    fn abort(self) {}
}

pub struct Handler<T:EventMachine> {
    slab: Slab<T>,
}

impl<T:EventMachine> mio::Handler for Handler<T> {
    type Message = Notify<T>;
    type Timeout = ();
    fn ready(&mut self, _eloop: &mut EventLoop<Self>,
        token: Token, events: EventSet)
    {
        self.slab.replace_with(token, |fsm| {
            fsm.ready(events)
        }).ok();  // Spurious events are ok in mio
    }

    fn notify(&mut self, eloop: &mut EventLoop<Self>, msg: Notify<T>) {
        use self::Notify::*;
        match msg {
            NewMachine(fsm) => {
                // This is so complex because of limitations of Slab
                match self.slab.insert(fsm) {
                    Ok(tok) => {
                        self.slab.replace_with(tok, |mut fsm| {
                            match fsm.register(tok, eloop) {
                                Ok(()) => Some(fsm),
                                Err(_) => {
                                    fsm.abort();
                                    None
                                }
                            }
                        }).unwrap();
                    }
                    Err(fsm) => {
                        fsm.abort();
                    }
                }
            }
        }
    }
}

