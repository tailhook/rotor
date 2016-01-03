=====
Rotor
=====

:Status: Alpha
:Examples: `TCP echo server`_, `TCP client (telnet)`_

.. _TCP echo server: https://github.com/tailhook/rotor/blob/master/examples/tcp_echo_server.rs
.. _TCP client (telnet): https://github.com/tailhook/rotor/blob/master/examples/telnet.rs

The mio-based framework for rust for doing I/O in simple and composable way.

The rotor core (this crate) basically consists of:

* An event loop handler (in terms of mio) which turns mio event into
  event to specific state machine
* A Future type which allows communication between state machines in safe
  and efficient way
* A simple way to combine multiple libraries (e.g. multiple protocol handlers)
  into single mio event loop

At the end of the day, rotor is the minimalistic core for making composable
libraries on top. It's less than 0.5KLoC.

You are expected to use some higher level abstraction most of the time.
For example, you should use stream abstraction (yet to be implemented) for
making TCP protocol parser.


Resources
=========

**Both are rather historical at the moment**

* `Asynchronous IO in Rust <https://medium.com/@paulcolomiets/asynchronous-io-in-rust-36b623e7b965>`_
  (random design notes about this library)
* `Asynchronous IO in Rust (part II) <https://medium.com/@paulcolomiets/async-io-for-rust-part-ii-33b9a7274e67>`_
* `Async IO in Rust (part III) <https://medium.com/@paulcolomiets/async-io-in-rust-part-iii-cbfd10f17203>`_


Benchmarks
==========

These benchmarks are based on **old version of** `this example`_. Hopefully
we will get updated benchmarks soon.

.. _this example: https://github.com/tailhook/rotor-http/blob/master/examples/hello_world_server.rs

Just few micro-benchmarks to show that framework has a decent peformance.

The performance on the few years old laptop (i7-3517U CPU @ 1.90GHz)::

    > wrk -t2 -c 400 http://localhost:8888/
    Running 10s test @ http://localhost:8888/
      2 threads and 400 connections
      Thread Stats   Avg      Stdev     Max   +/- Stdev
        Latency    11.19ms   18.03ms 627.44ms   99.54%
        Req/Sec    19.66k     1.76k   21.93k    81.00%
      391170 requests in 10.01s, 32.83MB read
    Requests/sec:  39071.42
    Transfer/sec:      3.28MB

Performance on newer desktop class CPU (i7-4790K CPU @ 4.00GHz)::

    > ./wrk -t 2 -c 400 http://127.0.0.1:8888
    Running 10s test @ http://127.0.0.1:8888
      2 threads and 400 connections
      Thread Stats   Avg      Stdev     Max   +/- Stdev
        Latency     2.24ms    1.56ms 126.94ms   99.91%
        Req/Sec    91.35k     2.27k   93.76k    98.00%
      1818133 requests in 10.00s, 152.58MB read
    Requests/sec: 181781.96
    Transfer/sec:     15.26MB

Note: both benchmarks are run on **single threaded** server.

The benchmarks are too early (not a full implementation of HTTP), so no
comparison bencmarks listed here.


