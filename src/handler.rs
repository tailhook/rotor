use std::io::Error;

use mio::{self, EventLoop, Token, EventSet};
use mio::util::Slab;


pub enum Notify<T> {
    NewMachine(T),
}

pub trait EventMachine<C>: Send + Sized {

    /// Socket readiness notification
    fn ready(self, events: EventSet, context: &mut C) -> Option<Self>;

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
    /// fails (which presumably means can't add socket to epoll). Or when
    /// sending machine fails (which means mio notify queue is full
    fn abort(self) {
        error!("Connection aborted because of lack of resources");
    }
}

pub struct Handler<C, T:EventMachine<C>> {
    slab: Slab<T>,
    context: C,
}

impl<C, T:EventMachine<C>> Handler<C, T> {
    pub fn new(context: C) -> Handler<C, T> {
        // TODO(tailhook) create default config from the ulimit data instead
        // of using real defaults
        Handler {
            slab: Slab::new(4096),
            context: context,
        }
    }
}

impl<C, T:EventMachine<C>> mio::Handler for Handler<C, T> {
    type Message = Notify<T>;
    type Timeout = ();
    fn ready(&mut self, _eloop: &mut EventLoop<Self>,
        token: Token, events: EventSet)
    {
        let ref mut ctx = self.context;
        self.slab.replace_with(token, |fsm| {
            fsm.ready(events, ctx)
        }).ok();  // Spurious events are ok in mio
    }

    fn notify(&mut self, eloop: &mut EventLoop<Self>, msg: Self::Message) {
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

