extern crate mio;
extern crate rotor;

use std::io::{Write, stderr};

use mio::{EventSet, PollOpt, TryRead, TryWrite};
use mio::tcp::{TcpListener, TcpStream};
use rotor::{Machine, Response, Scope};


struct Context;

enum Echo {
    Server(TcpListener),
    Connection(TcpStream),
}

impl Echo {
    fn accept(self) -> Response<Echo> {
        match self {
            Echo::Server(sock) => {
                match sock.accept() {
                    Ok(Some((conn, _))) => {
                        Response::spawn(Echo::Server(sock),
                                        Echo::Connection(conn))
                    }
                    Ok(None) => {
                        Response::ok(Echo::Server(sock))
                    }
                    Err(e) => {
                        writeln!(&mut stderr(), "Error: {}", e).ok();
                        Response::ok(Echo::Server(sock))
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Machine<Context> for Echo {
    fn register(self, scope: &mut Scope<Context>) -> Response<Self> {
        match self {
            Echo::Server(sock) => {
                scope.register(&sock, EventSet::readable(), PollOpt::edge())
                    .unwrap();
                Response::ok(Echo::Server(sock))
            }
            Echo::Connection(sock) => {
                scope.register(&sock, EventSet::readable(), PollOpt::level())
                    .unwrap();
                Response::ok(Echo::Connection(sock))
            }
        }
    }
    fn ready(self, _events: EventSet, _scope: &mut Scope<Context>)
        -> Response<Self>
    {
        match self {
            me @ Echo::Server(..) => me.accept(),
            Echo::Connection(mut sock) => {
                let mut data = [0u8; 1024];
                match sock.try_read(&mut data) {
                    Err(e) => {
                        writeln!(&mut stderr(), "read: {}", e).ok();
                        Response::done()
                    }
                    Ok(Some(x)) => {
                        match sock.try_write(&data[..x]) {
                            Ok(_) => {
                                // this is example so we don't care if not all
                                // (or none at all) bytes are written
                                Response::ok(Echo::Connection(sock))
                            }
                            Err(e) => {
                                writeln!(&mut stderr(), "write: {}", e).ok();
                                Response::done()
                            }
                        }
                    }
                    Ok(None) => {
                        Response::ok(Echo::Connection(sock))
                    }
                }
            }
        }
    }
    fn spawned(self, _scope: &mut Scope<Context>) -> Response<Self>
    {
        match self {
            me @ Echo::Server(..) => me.accept(),
            _ => unreachable!(),
        }
    }
    fn timeout(self, _scope: &mut Scope<Context>) -> Response<Self> {
        unreachable!();
    }
    fn wakeup(self, _scope: &mut Scope<Context>) -> Response<Self> {
        unreachable!();
    }
}

fn main() {
    let mut event_loop = mio::EventLoop::new().unwrap();
    let mut handler = rotor::Handler::new(Context, &mut event_loop);
    let lst = TcpListener::bind(&"127.0.0.1:3000".parse().unwrap()).unwrap();
    handler.add_root(&mut event_loop, Echo::Server(lst));
    event_loop.run(&mut handler).unwrap();
}
