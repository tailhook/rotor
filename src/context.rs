use mio::Sender;

use handler::Notify;


pub trait AsyncContext<M:Send> {
    fn async_channel<'x>(&'x self) -> &'x Sender<Notify<M>>;
    fn async_add_machine(&mut self, m: M) -> Result<(), M> {
        use mio::NotifyError::*;
        match self.async_channel().send(Notify::NewMachine(m)) {
            Ok(()) => Ok(()),
            Err(Io(e)) => {
                // We would probably do something better here, but mio doesn't
                // give us a message. But anyway it's probably never happen
                panic!("Io error when sending notify: {}", e);
            }
            Err(Full(Notify::NewMachine(m))) => Err(m),
            Err(Closed(_)) => {
                // It should never happen because we usually send from the
                // inside of a main loop
                panic!("Sending to closed channel. Main loop is already shut \
                    down");
            }
        }
    }
}
