# FAQ

## What is a State Machine?

You probably know [the theory][wiki:sm]. In this docs when we refer to state
machine we refer to a type (most of the time the enum) that implements some
trait designed according to the rules below. There is some [introductory
article about why state machines are designed that way][aio-article].

Rules: TBD

[wiki:sm]: https://en.wikipedia.org/wiki/State_machine
[aio-article]: https://medium.com/@paulcolomiets/asynchronous-io-in-rust-36b623e7b965

## What's difference between Context and Scope?

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
