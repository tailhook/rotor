=====
Rotor
=====

:Status: Proof of Concept
:Examples: `in rotor-http library`__

.. __: https://github.com/tailhook/rotor-http/tree/master/examples

**This is 0.4.x version of the library it's very stripped, and has not proven
to work yet. Stay tuned!**

The mio-based framework for rust for doing I/O in simple and composable way.

Features:

* Based on hierarchical state machine concept
* Ownership semantics for state machines allows to write
  without memory/resource leaks
* Easy to combine multiple libraries into single mio event loop

Resources
=========

* `Asynchronous IO in Rust`__ (random design notes about this library)
* `Asynchronous IO in Rust (part II)`__

.. __: https://medium.com/@paulcolomiets/asynchronous-io-in-rust-36b623e7b965
.. __: https://medium.com/@paulcolomiets/async-io-for-rust-part-ii-33b9a7274e67

Benchmarks
==========

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


