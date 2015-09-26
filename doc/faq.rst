===
FAQ
===

Library Design
==============

What's difference between Context and Scope?
--------------------------------------------

Here is rule of thumb:

* `Context` is a structure that contains global state for the application
* `Scope` is a structure that contains local per-event data

More details:

* The lifetime of a `Context` outlives the main loop, while the lifetime of a
  `Scope` is just a single event handler (it's stored on stack)
* `Context` is a structure that has some trait(s) implemented by each
  library/application, and some traits required by some state machines
* `Scope` is just a set of pointers to main loop structures and similar that
  can't be stored globally
