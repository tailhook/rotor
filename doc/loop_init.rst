.. highlight:: rust

===================
Loop Initialization
===================


Overview
========


Loop initialization has two stages. First is created by::

    let loop_creator = try!(rotor::Loop::new());

And the second is created by::

    let loop_instance = loop_creator.instantiate(context)

Then you can run the loop::

    try!(loop_instance.run());

As you can see the ``loop_creator.instantiate(..)`` takes a context for the
instantiation. This *is* the **key difference** between two stages.

There is a shortcut if you want to skip second stage of initialization::

    let loop_creator = try!(rotor::Loop::new());
    try!(loop_creator.run(context));


Adding State Machines
=====================

To have something useful of main loop you need to add a state machine to
it. State machine initialization is done via ``add_machine_with`` method::

    try!(loop_creator.add_machine_with(|scope| {
        Ok(Tcp::new(addr, scope))
    }));

And in loop instance there is similar method::

    try!(loop_instance.add_machine_with(|scope| {
        Ok(Tcp::new(addr, scope))
    }));

The difference is in the signature of the function::

    impl Loop {
        fn add_machine_with<F>(&mut self, fun: F)
            -> Result<(), SpawnError<()>>
            where F: FnOnce(&mut EarlyScope) -> Result<M, Box<Error>>;
    }
    impl LoopInstance {
        fn add_machine_with<F>(&mut self, fun: F)
            -> Result<(), SpawnError<()>>
            where F: FnOnce(&mut Scope<C>) -> Result<M, Box<Error>>;
    }

As you can see the only difference is that loop creator gets ``EarlyScope``
as an argument and latter gets ``Scope<Context>`` as an argument::

1. Both have ``GenericScope`` implementation, so you can have constructors
   generic over the scope type
2. ``Scope`` dereferences to the context while ``EarlyScope`` does not

*Thats it*. But in reality it's important. For example, rotor-dns_ creates
a pair: a state machine and a resolver object. State machine is just added
to a loop, but you may want to put resolver object to a context. For example::

    extern crate rotor_dns;

    let resolver_opt = None;
    try!(loop_creator.add_machine_with(|scope| {
        let (res, fsm) = try!(rotor_dns::create_resolver(scope, cfg));
        resolver_opt = Some(res);
        Ok(fsm)
    }));
    let resolver = resolver_opt.unwrap();
    let mut loop_instance = loop_creator.instantiate(Context {
        dns: resolver,
    });
    loop_instance.add_machine_with(..)

With rotor-tools_ the code is simplified to::

    extern crate rotor_dns;
    extern crate rotor_tools;
    use rotor_tools::LoopExt;  // The trait with helper functions

    let resolver = try!(loop_creator.add_and_fetch(|scope| {
        rotor_dns::create_resolver(scope, cfg)
    }));
    let mut loop_instance = loop_creator.instantiate(Context {
        dns: resolver,
    });
    loop_instance.add_machine_with(..)

.. _rotor-dns: http://github.com/tailhook/rotor-dns/
.. _rotor-tools: http://github.com/tailhook/rotor-tools/



