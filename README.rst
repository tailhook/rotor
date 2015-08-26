=====
Rotor
=====

:Status: Proof of Concept
:Examples: `in rotor-http library`__

.. __: https://github.com/tailhook/rotor-http/tree/master/examples

The mio-based framework for rust for doing I/O in simple and composable way.

Features:

* Based on hierarchical state machine concept
* Ownership semantics for state machines allows to write
  without memory/resource leaks
* Easy to combine multiple libraries into single mio event loop

