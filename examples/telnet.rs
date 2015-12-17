extern crate mio;
extern crate nix;
#[macro_use] extern crate rotor;
extern crate argparse;
extern crate void;

use std::io::{Write, stderr, stdout};
use std::os::unix::io::FromRawFd;

use void::Void;
use mio::{EventSet, PollOpt, TryRead, TryWrite};
use mio::tcp::{TcpStream};
use mio::unix::{UnixStream};
use nix::fcntl::{fcntl, FcntlArg, O_NONBLOCK};
use rotor::{Machine, Response, Scope}; // Compose2
use argparse::{ArgumentParser, Store};


struct Context;

enum Tcp {
    Connecting(TcpStream),
    Reading(TcpStream),
}

struct Stdin {
    input: UnixStream,
    output: TcpStream,
}

//You can use a type
//type Composed = Compose2<Tcp, Stdin>;

//Or alternatively use macro
rotor_compose!(enum Composed/CCreator/CError <Context> {
    Tcp(Tcp),
    Stdin(Stdin),
});

// ^^ note that enum names are different, you need to fix them below

impl Tcp {
    fn new(sock: TcpStream, scope: &mut Scope<Context>) -> Tcp
    {
        scope.register(&sock, EventSet::writable(), PollOpt::level())
            .unwrap();
        Tcp::Connecting(sock)
    }
}

impl Machine<Context> for Tcp {
    type Creator = Void;
    fn ready(self, _events: EventSet, scope: &mut Scope<Context>)
        -> Response<Self, Void>
    {
        match self {
            Tcp::Connecting(sock) => {
                scope.reregister(&sock, EventSet::readable(), PollOpt::level())
                    .unwrap();
                Response::ok(Tcp::Reading(sock))
            }
            Tcp::Reading(mut sock) => {
                let mut data = [0u8; 1024];
                match sock.try_read(&mut data) {
                    Err(e) => {
                        writeln!(&mut stderr(), "read: {}", e).ok();
                        Response::done()
                    }
                    Ok(Some(x)) => {
                        // We don't check the result, for making example
                        // super-simple.
                        stdout().write(&data[..x]).ok();
                        Response::ok(Tcp::Reading(sock))
                    }
                    Ok(None) => {
                        Response::ok(Tcp::Reading(sock))
                    }
                }
            }
        }
    }
    fn spawned(self, _scope: &mut Scope<Context>) -> Response<Self, Void>
    {
        unreachable!();
    }
    fn timeout(self, _scope: &mut Scope<Context>) -> Response<Self, Void> {
        unreachable!();
    }
    fn wakeup(self, _scope: &mut Scope<Context>) -> Response<Self, Void> {
        unreachable!();
    }
}

impl Stdin {
    fn new(dest: TcpStream, scope: &mut Scope<Context>) -> Stdin
    {
        let stdin = unsafe { UnixStream::from_raw_fd(0) };
        scope.register(&stdin, EventSet::writable(), PollOpt::level())
            .unwrap();
        Stdin {
            input: stdin,
            output: dest,
        }
    }
}

impl Machine<Context> for Stdin {
    type Creator = Void;
    fn ready(mut self, _events: EventSet, _scope: &mut Scope<Context>)
        -> Response<Self, Void>
    {
        let mut data = [0u8; 1024];
        match self.input.try_read(&mut data) {
            Err(e) => {
                writeln!(&mut stderr(), "read: {}", e).ok();
                return Response::done()
            }
            Ok(Some(x)) => {
                // We don't check the result, for making example
                // super-simple.
                match self.output.try_write(&data[..x]) {
                    Ok(_) => {
                        // this is example so we don't care if not all
                        // (or none at all) bytes are written
                    }
                    Err(e) => {
                        writeln!(&mut stderr(), "write: {}", e).ok();
                        return Response::done()
                    }
                }
            }
            Ok(None) => { }
        }
        Response::ok(self)
    }
    fn spawned(self, _scope: &mut Scope<Context>) -> Response<Self, Void>
    {
        unreachable!();
    }
    fn timeout(self, _scope: &mut Scope<Context>) -> Response<Self, Void> {
        unreachable!();
    }
    fn wakeup(self, _scope: &mut Scope<Context>) -> Response<Self, Void> {
        unreachable!();
    }
}

fn main() {
    let mut host = "127.0.0.1".to_string();
    let mut port = 3000u16;
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Telnet-like example for mio.

            Note this relies on the ability to always do successful writes to
            the socket (which is true for small interactive sessions). Do not
            use it as a real telnet client.
            ");
        ap.refer(&mut host).add_argument("ip", Store, "
            The IP addresss to connect to. No name resolution here.");
        ap.refer(&mut port).add_argument("port", Store, "
            Port to connect to. Default is 3000 which is the port of
            tcp-echo-server example.");
        ap.parse_args_or_exit();
    }

    let mut event_loop = mio::EventLoop::new().unwrap();
    let mut handler = rotor::Handler::new(Context, &mut event_loop);

    let conn = TcpStream::connect(
        // Any better way for current stable rust?
        &format!("{}:{}", host, port).parse().unwrap()).unwrap();
    let conn2 = conn.try_clone().unwrap();

    // We clone output socket so we don't need to communicate between sockets
    // This isn't a good idea for the real work
    fcntl(0, FcntlArg::F_SETFL(O_NONBLOCK)).expect("fcntl");

    let conn = handler.add_machine_with(&mut event_loop, |scope| {
        Composed::Tcp(Tcp::new(conn))
    });
    let stdio = handler.add_machine_with(&mut event_loop, |scope| {
        Composed::Stdin(Stdin::new(conn2))
    });
    event_loop.run(&mut handler).unwrap();
}
