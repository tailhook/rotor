.. _ecosystem:

=========
Ecosystem
=========


Libraries
=========

* `rotor-tools <https://crates.io/crates/rotor-tools/>`_ -- a collection of
  small convenience utilities
* `rotor-test <https://crates.io/crates/rotor-test/>`_ -- a collection of
  utilities for writing unit tests
* `rotor-stream <https://crates.io/crates/rotor-stream/>`_ -- an abstraction for
  writing protocols which use TCP or Unix stream sockets
* `rotor-carbon <https://crates.io/crates/rotor-carbon/>`_ -- implementation of
  the `carbon <http://graphite.wikidot.com/>`_ protocol (more known as graphite)
* `rotor-dns <https://crates.io/crates/rotor-dns/>`_ -- DNS support for rotor
* `rotor-http <https://crates.io/crates/rotor-http/>`_ -- HTTP server and client
  implementation
* `rotor-redis <https://github.com/tailhook/rotor-redis/>`_ -- redis client
  implementation
* `hyper <https://github.com/hyperium/hyper/>`_
  the implementation fo HTTP protocol added to hyper itself
* `rotor-capnp <https://github.com/0x1997/rotor-capnp>`_ -- implementation
  of Cap'n'Proto protocol


Applications
============

* `Kinglet <https://github.com/pyfisch/kinglet>`_ -- a HTTP server
* `basic-http-server <https://github.com/brson/basic-http-server>`_ -- also a
  HTTP server


Other
=====

* `stator <https://github.com/tailhook/stator>`_ -- a wrapper around foreign
  function interface (FFI) for various rotor libraries that allows
  dispatching them from scripting languages; thus offloading asynchronous
  and protocol parsing work to rotor that is put in separate thread; so
  rust code is running in parallel to the scripting language interpreter.
