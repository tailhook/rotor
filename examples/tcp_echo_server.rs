extern crate rotor;

use std::io::{Write, stderr};

use rotor::{EventSet, PollOpt, Loop, Config, Void};
use rotor::mio::deprecated::{TryRead, TryWrite};
use rotor::mio::tcp::{TcpListener, TcpStream};
use rotor::{Machine, Response, EarlyScope, Scope};


struct Context;

enum Echo {
    Server(TcpListener),
    Connection(TcpStream),
}

impl Echo {
    pub fn new(sock: TcpListener, scope: &mut EarlyScope)
        -> Response<Echo, Void>
    {
        scope.register(&sock, EventSet::readable(), PollOpt::edge())
            .unwrap();
        Response::ok(Echo::Server(sock))
    }
    fn accept(self) -> Response<Echo, TcpStream> {
        match self {
            Echo::Server(sock) => {
                match sock.accept() {
                    Ok((conn, _)) => {
                        Response::spawn(Echo::Server(sock), conn)
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
        -> Response<Self, Void>
    {
        scope.register(&conn, EventSet::readable(), PollOpt::level())
            .unwrap();
        Response::ok(Echo::Connection(conn))
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
    let mut loop_creator = Loop::new(&Config::new()).unwrap();
    let lst = TcpListener::bind(&"127.0.0.1:3000".parse().unwrap()).unwrap();
    loop_creator.add_machine_with(|scope| {
        Echo::new(lst, scope)
    }).unwrap();
    loop_creator.run(Context).unwrap();
}
