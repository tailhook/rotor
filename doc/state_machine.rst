.. highlight:: rust

===========================
Implementing State Machines
===========================

This guide is for **authors** of the protocols, *not* for **users** of the
protocols. Read it if you want to write an **new** protocol implementation on
top of raw ``rotor::Machine``. Otherwise consult on your protocol documenation
(probably good links are in :ref:`Ecosystem`)


Boilerplate
===========

This is just a blanket stub implementation I usually start with, filling
in methods one by one::

    extern crate rotor;
    extern crate void;

    use std::error::Error;
    use rotor::{Machine, EventSet, PollOpt, Scope, Response};
    use void::{unreachable, Void};

    impl<C> Machine for Fsm<C> {
        type Context = C;
        type Seed = Void;
        fn create(seed: Self::Seed, _scope: &mut Scope<C>)
            -> Result<Self, Box<Error>>
        {
            unreachable(seed)
        }
        fn ready(self, _events: EventSet, _scope: &mut Scope<C>)
            -> Response<Self, Self::Seed>
        {
            unimplemented!();
        }
        fn spawned(self, _scope: &mut Scope<C>) -> Response<Self, Self::Seed>
        {
            unimplemented!();
        }
        fn timeout(self, _scope: &mut Scope<C>) -> Response<Self, Self::Seed>
        {
            unimplemented!();
        }
        fn wakeup(self, _scope: &mut Scope<C>) -> Response<Self, Self::Seed>
        {
            unimplemented!();
        }
    }

There are two intricate things here:

1. We use ``void`` crate and ``void::Void`` type to denote that seed can't be
   created so ``create`` method is never called

   Keep the type ``void`` unless your machine spawns new state machines. And
   in the latter case it's advised to use some abstraction for state machine
   spawning.  There is an ``rotor_stream::Accept`` for accepting sockets, more
   to come.

2. Implementation should almost always use generic context (``impl<C>``) as
   only end application should know the exact layout of a context.

   You may limit the generic with some traits (``impl<C: HttpContext>``).

   Often, your state machine doesn't rely on context at all. Currently, this
   requires adding a ``PhantomData<*const C>`` marker to state machine.
   The marker_ is zero-sized, so it just a little bit of boring code.

.. _marker:: http://doc.rust-lang.org/std/marker/struct.PhantomData.html

