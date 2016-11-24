(function() {var implementors = {};
implementors["void"] = [];implementors["libc"] = [];implementors["nix"] = [];implementors["mio"] = ["impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='mio/tcp/struct.TcpStream.html' title='mio::tcp::TcpStream'>TcpStream</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='mio/tcp/struct.TcpListener.html' title='mio::tcp::TcpListener'>TcpListener</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='mio/udp/struct.UdpSocket.html' title='mio::udp::UdpSocket'>UdpSocket</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='mio/deprecated/unix/struct.Io.html' title='mio::deprecated::unix::Io'>Io</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='mio/deprecated/unix/struct.UnixSocket.html' title='mio::deprecated::unix::UnixSocket'>UnixSocket</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='mio/deprecated/unix/struct.UnixStream.html' title='mio::deprecated::unix::UnixStream'>UnixStream</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='mio/deprecated/unix/struct.UnixListener.html' title='mio::deprecated::unix::UnixListener'>UnixListener</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='mio/deprecated/unix/struct.PipeReader.html' title='mio::deprecated::unix::PipeReader'>PipeReader</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='mio/deprecated/unix/struct.PipeWriter.html' title='mio::deprecated::unix::PipeWriter'>PipeWriter</a>",];implementors["rotor"] = ["impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='rotor/mio/tcp/struct.TcpStream.html' title='rotor::mio::tcp::TcpStream'>TcpStream</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='rotor/mio/tcp/struct.TcpListener.html' title='rotor::mio::tcp::TcpListener'>TcpListener</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='rotor/mio/udp/struct.UdpSocket.html' title='rotor::mio::udp::UdpSocket'>UdpSocket</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='rotor/mio/deprecated/unix/struct.Io.html' title='rotor::mio::deprecated::unix::Io'>Io</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='rotor/mio/deprecated/struct.UnixSocket.html' title='rotor::mio::deprecated::UnixSocket'>UnixSocket</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='rotor/mio/deprecated/struct.UnixStream.html' title='rotor::mio::deprecated::UnixStream'>UnixStream</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='rotor/mio/deprecated/struct.UnixListener.html' title='rotor::mio::deprecated::UnixListener'>UnixListener</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='rotor/mio/deprecated/struct.PipeReader.html' title='rotor::mio::deprecated::PipeReader'>PipeReader</a>","impl <a class='trait' href='https://doc.rust-lang.org/nightly/std/sys/ext/io/trait.IntoRawFd.html' title='std::sys::ext::io::IntoRawFd'>IntoRawFd</a> for <a class='struct' href='rotor/mio/deprecated/struct.PipeWriter.html' title='rotor::mio::deprecated::PipeWriter'>PipeWriter</a>",];

            if (window.register_implementors) {
                window.register_implementors(implementors);
            } else {
                window.pending_implementors = implementors;
            }
        
})()
