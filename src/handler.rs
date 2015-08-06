use mio::{self, EventLoop, Token, EventSet};
use mio::util::Slab;

pub trait EventMachine {
    fn ready(self, events: EventSet) -> Option<Self>;
}

pub struct Handler<T:EventMachine> {
    slab: Slab<T>,
}

impl<T:EventMachine> mio::Handler for Handler<T> {
    type Message = ();
    type Timeout = ();
    fn ready(&mut self, _eloop: &mut EventLoop<Self>,
        token: Token, events: EventSet)
    {
        self.slab.replace_with(token, |fsm| {
            fsm.ready(events)
        }).ok();  // Spurious events are ok in mio
    }
}

