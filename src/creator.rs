use std::io;

use mio::EventLoop;
use mio::util::Slab;
use void::{Void, unreachable};

use config::{create_slab, create_loop};
use handler::{Handler, create_handler, set_timeout_opt};
use scope::{early_scope, EarlyScope, scope, Scope};
use {Machine, Config, SpawnError, Timeout, Time, Response};
use SpawnError::NoSlabSpace;
use response::decompose;


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
/// The [the guide] for more information.
///
/// [the guide]: http://rotor.readthedocs.org/en/latest/loop_init.html
pub struct LoopCreator<M: Machine> {
    slab: Slab<(Option<(Timeout, Time)>, M)>,
    mio: EventLoop<Handler<M>>,
}
/// Second stage of loop creation
///
/// See the docs of `LoopCreator` or [the guide] for more information.
///
/// [the guide]: http://rotor.readthedocs.org/en/latest/loop_init.html
pub struct LoopInstance<M: Machine> {
    mio: EventLoop<Handler<M>>,
    handler: Handler<M>,
}

impl<M: Machine> LoopCreator<M> {
    pub fn new(cfg: &Config) -> Result<LoopCreator<M>, io::Error> {
        let slab = create_slab(&cfg);
        let eloop = try!(create_loop(&cfg));
        Ok(LoopCreator {
            slab: slab,
            mio: eloop,
        })
    }

    pub fn add_machine_with<F>(&mut self, fun: F) -> Result<(), SpawnError<()>>
        where F: FnOnce(&mut EarlyScope) -> Response<M, Void>
    {
        let ref mut chan = self.mio.channel();
        let ref mut mio = self.mio;
        let res = self.slab.insert_with(|token| {
            let ref mut scope = early_scope(token, chan, mio);
            let (mach, void, timeout) =  decompose(token, fun(scope));
            void.map(|x| unreachable(x));
            let m = mach.expect("You can't return Response::done() \
                  from Machine::create() until new release of slab crate. \
                  (requires insert_with_opt)");
            let to = set_timeout_opt(timeout, scope);
            (to, m)
        });
        if res.is_some() {
            Ok(())
        } else {
            // TODO(tailhook) propagate error from state machine construtor
            Err(NoSlabSpace(()))
        }
    }

    pub fn instantiate(self, context: M::Context) -> LoopInstance<M> {
        let LoopCreator { slab, mio } = self;
        let handler = create_handler(slab, context, mio.channel());
        LoopInstance { mio: mio, handler: handler }
    }

    pub fn run(self, context: M::Context) -> Result<(), io::Error> {
        self.instantiate(context).run()
    }
}

impl<M: Machine> LoopInstance<M> {

    pub fn add_machine_with<F>(&mut self, fun: F) -> Result<(), SpawnError<()>>
        where F: FnOnce(&mut Scope<M::Context>) -> Response<M, Void>
    {
        self.handler.add_machine_with(&mut self.mio, fun)
    }

    pub fn run(mut self) -> Result<(), io::Error> {
        let ref mut handler = self.handler;
        let ref mut mio = self.mio;
        mio.run(handler)
    }
}
