========
Glossary
========

state machine
    You probably know `the theory`__. In this docs when we refer to
    state machine we refer to a type (most of the time the enum) that
    implements some trait designed according to the rules below. There is some
    `introductory article about why state machines are designed that
    way`__.

    State machine implements at least abstract ``rotor::base::Machine`` trait.
    But there are also more state machine traits that are more concrete.

    Rules: TBD

.. __: https://en.wikipedia.org/wiki/State_machine
.. __: https://medium.com/@paulcolomiets/asynchronous-io-in-rust-36b623e7b965

.. _child state machine:

child state machine
    Often one state machine calls an action from another state machine. The
    one that calls actions is a **parent**. The one that receives actions
    is called **child**. The parent state machine usually also owns the parent
    state machine (means that when parent is shut down, the all the children
    too).

    There might be multiple child state machines when the protocol allows
    multiple underlying requests/substreams/whatever to be mixed and used
    simultaneously

parent state machine
    See `child state machine`_

