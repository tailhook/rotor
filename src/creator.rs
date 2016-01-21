use std::io;
use std::error::Error;

use mio::EventLoop;
use mio::util::Slab;

use config::{create_slab, create_loop};
use handler::{create_handler};
use scope::{early_scope, EarlyScope, Scope};
use {Machine, Config, Handler, SpawnError};
use SpawnError::NoSlabSpace;


/// An object that is used to construct a loop
///
/// The purpose of the object is to shorten the boilerplate to create
/// an event loop.
///
/// The second purpose is to create the loop and state machines
/// before Context is initialized. This is useful when you want to put
/// `Notifier` objects of state machines into the context.
///
/// You can create a loop either right away:
///
/// ```ignore
/// use rotor::{Loop, Config};
///
/// let mut lc = Loop::new(&Config::new()).unwrap();
/// loop_creator.add_machine_with(|scope| {
///     // The scope here is the `EarlyScope` (no context)
///     Ok(CreateMachine(x))
/// }).unwrap();
/// assert!(conn.is_ok());
/// lc.run(context).unwrap()
/// ```
///
/// Or if you can create it in two stages:
///
/// ```ignore
/// let lc = Loop::new(&Config::new()).unwrap();
/// loop_creator.add_machine_with(|scope| {
///     // The scope here is the `EarlyScope`
///     Ok(StateMachine1(scope))
/// }).unwrap();
/// let mut inst = lc.instantiate(context);
/// loop_creator.add_machine_with(|scope| {
///     // The scope here is the real `Scope<C>`
///     Ok(StateMachine2(scope))
/// }).unwrap();
/// inst.run().unwrap()
/// ```
///
///
pub struct LoopCreator<C, M: Machine<Context=C>> {
    slab: Slab<M>,
    mio: EventLoop<Handler<C, M>>,
}
pub struct LoopInstance<C, M: Machine<Context=C>> {
    mio: EventLoop<Handler<C, M>>,
    handler: Handler<C, M>,
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

    pub fn add_machine_with<F>(&mut self, fun: F) -> Result<(), SpawnError<()>>
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
            Err(NoSlabSpace(()))
        }
    }

    pub fn instantiate(self, context: C) -> LoopInstance<C, M> {
        let LoopCreator { slab, mio } = self;
        let handler = create_handler(slab, context, mio.channel());
        LoopInstance { mio: mio, handler: handler }
    }

    pub fn run(self, context: C) -> Result<(), io::Error> {
        self.instantiate(context).run()
    }
}

impl<C, M: Machine<Context=C>> LoopInstance<C, M> {

    pub fn add_machine_with<F>(&mut self, fun: F) -> Result<(), SpawnError<()>>
        where F: FnOnce(&mut Scope<C>) -> Result<M, Box<Error>>
    {
        let ref mut handler = self.handler;
        let ref mut mio = self.mio;
        handler.add_machine_with(mio, fun)
    }

    pub fn run(mut self) -> Result<(), io::Error> {
        let ref mut handler = self.handler;
        let ref mut mio = self.mio;
        mio.run(handler)
    }
}
