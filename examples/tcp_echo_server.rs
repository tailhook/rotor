extern crate rotor;

use std::io::{Write, stderr};
use std::error::Error;

use rotor::{EventSet, PollOpt, EventLoop};
use rotor::mio::{TryRead, TryWrite};
use rotor::mio::tcp::{TcpListener, TcpStream};
use rotor::{Machine, Response, Scope};


struct Context;

enum Echo {
    Server(TcpListener),
    Connection(TcpStream),
}

impl Echo {
    pub fn new(sock: TcpListener, scope: &mut Scope<Context>) -> Echo {
        scope.register(&sock, EventSet::readable(), PollOpt::edge())
            .unwrap();
        Echo::Server(sock)
    }
    fn accept(self) -> Response<Echo, TcpStream> {
        match self {
            Echo::Server(sock) => {
                match sock.accept() {
                    Ok(Some((conn, _))) => {
                        Response::spawn(Echo::Server(sock), conn)
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

impl Machine for Echo {
    type Context = Context;
    type Seed = TcpStream;

    fn create(conn: TcpStream, scope: &mut Scope<Context>)
        -> Result<Self, Box<Error>>
    {
        scope.register(&conn, EventSet::readable(), PollOpt::level())
            .unwrap();
        Ok(Echo::Connection(conn))
    }

    fn ready(self, _events: EventSet, _scope: &mut Scope<Context>)
        -> Response<Self, TcpStream>
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
                    Ok(Some(0)) => {
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
    fn spawned(self, _scope: &mut Scope<Context>) -> Response<Self, TcpStream>
    {
        match self {
            me @ Echo::Server(..) => me.accept(),
            _ => unreachable!(),
        }
    }
    fn timeout(self, _scope: &mut Scope<Context>)
        -> Response<Self, TcpStream>
    {
        unreachable!();
    }
    fn wakeup(self, _scope: &mut Scope<Context>)
        -> Response<Self, TcpStream>
    {
        unreachable!();
    }
}

fn main() {
    let mut event_loop = EventLoop::new().unwrap();
    let mut handler = rotor::Handler::new(Context, &mut event_loop);
    let lst = TcpListener::bind(&"127.0.0.1:3000".parse().unwrap()).unwrap();
    let ok = handler.add_machine_with(&mut event_loop, |scope| {
        Ok(Echo::new(lst, scope))
    }).is_ok();
    assert!(ok);
    event_loop.run(&mut handler).unwrap();
}
