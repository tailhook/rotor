use std::io;
use std::error::Error;

use mio::EventLoop;
use mio::util::Slab;

use config::{create_slab, create_loop};
use handler::{create_handler, NoSlabSpace};
use scope::{early_scope, EarlyScope};
use {Machine, Config, Handler};


/// An object that is used to construct a loop
///
/// The purpose of the object is to shorten the boilerplate to create
/// an event loop.
///
/// The second purpose is to create the loop and state machines
/// before Context is initialized. This is useful when you want to put
/// `Notifier` objects of state machines into the context.
pub struct LoopCreator<C, M: Machine<Context=C>> {
    slab: Slab<M>,
    mio: EventLoop<Handler<C, M>>,
}

impl<C, M: Machine<Context=C>> LoopCreator<C, M> {
    pub fn new(cfg: &Config) -> Result<LoopCreator<C, M>, io::Error> {
        let slab = create_slab(&cfg);
        let eloop = try!(create_loop(&cfg));
        Ok(LoopCreator {
            slab: slab,
            mio: eloop,
        })
    }

    pub fn add_machine_with<F>(&mut self, fun: F) -> Result<(), Box<Error>>
        where F: FnOnce(&mut EarlyScope) -> Result<M, Box<Error>>
    {
        let ref mut chan = self.mio.channel();
        let ref mut mio = self.mio;
        let res = self.slab.insert_with(|token| {
            let ref mut scope = early_scope(token, chan, mio);
            match fun(scope) {
                Ok(x) => x,
                Err(_) => {
                // TODO(tailhook) when Slab::insert_with_opt() lands, fix it
                    panic!("Unimplemented: Slab::insert_with_opt");
                }
            }
        });
        if res.is_some() {
            Ok(())
        } else {
            Err(Box::new(NoSlabSpace(())))
        }
    }

    pub fn run(self, context: C) -> Result<(), io::Error> {
        let LoopCreator { slab, mut mio } = self;
        let mut handler = create_handler(slab, context, mio.channel());
        mio.run(&mut handler)
    }
}

