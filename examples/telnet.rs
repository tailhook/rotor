extern crate mio;
extern crate nix;
#[macro_use] extern crate rotor;
extern crate argparse;

use std::io::{Write, stderr, stdout};
use std::os::unix::io::FromRawFd;

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
rotor_compose!(enum Composed <Context> {
    Tcp(Tcp),
    Stdin(Stdin),
});

// ^^ note that enum names are different, you need to fix them below

impl Machine<Context> for Tcp {
    fn register(self, scope: &mut Scope<Context>) -> Response<Self> {
        match self {
            Tcp::Connecting(sock) => {
                scope.register(&sock, EventSet::writable(), PollOpt::level())
                    .unwrap();
                Response::ok(Tcp::Connecting(sock))
            }
            _ => unreachable!(),
        }
    }
    fn ready(self, _events: EventSet, scope: &mut Scope<Context>)
        -> Response<Self>
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
    fn spawned(self, _scope: &mut Scope<Context>) -> Response<Self>
    {
        unreachable!();
    }
    fn timeout(self, _scope: &mut Scope<Context>) -> Response<Self> {
        unreachable!();
    }
    fn wakeup(self, _scope: &mut Scope<Context>) -> Response<Self> {
        unreachable!();
    }
}

impl Machine<Context> for Stdin {
    fn register(self, scope: &mut Scope<Context>) -> Response<Self> {
        scope.register(&self.input, EventSet::writable(), PollOpt::level())
            .unwrap();
        Response::ok(self)
    }
    fn ready(mut self, _events: EventSet, _scope: &mut Scope<Context>)
        -> Response<Self>
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
    fn spawned(self, _scope: &mut Scope<Context>) -> Response<Self>
    {
        unreachable!();
    }
    fn timeout(self, _scope: &mut Scope<Context>) -> Response<Self> {
        unreachable!();
    }
    fn wakeup(self, _scope: &mut Scope<Context>) -> Response<Self> {
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

    let lst = TcpStream::connect(
        // Any better way for current stable rust?
        &format!("{}:{}", host, port).parse().unwrap()).unwrap();

    // We clone output socket so we don't need to communicate between sockets
    // This isn't a good idea for the real work
    fcntl(0, FcntlArg::F_SETFL(O_NONBLOCK)).expect("fcntl");
    let stdin = Stdin {
        input: unsafe { UnixStream::from_raw_fd(0) },
        output: lst.try_clone().unwrap(),
    };

    handler.add_root(&mut event_loop, Composed::Tcp(Tcp::Connecting(lst)));
    handler.add_root(&mut event_loop, Composed::Stdin(stdin));
    event_loop.run(&mut handler).unwrap();
}
