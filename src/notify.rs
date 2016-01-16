use mio::{Token, Sender};

use handler::{Notify};

quick_error! {
    /// Error when waking up a connection
    ///
    /// In most cases it's okay to panic on this error
    #[derive(Debug)]
    pub enum WakeupError {
        /// I/O error when sending data to internal pipe
        ///
        /// We discard the io error as there no practical reason for this
        /// error to occur
        Io {
            description("I/O error happened when trying to wake up")
        }
        /// The pipe is full, the useful thing to do is configure longer
        /// queue in mio loop. Or alternatively, send less messages.
        Full {
            description("The notification pipe is full. \
                         You may want to increase it's size")
        }
        /// The notification queue is closed. Probably event loop is shut down
        Closed {
            description("Notification queue is close")
        }
    }
}


/// The object used to wakeup unrelated state machine
///
/// You may use a notifiers between multiple threads
#[derive(Clone, Debug)]
pub struct Notifier {
    token: Token,
    channel: Sender<Notify>,
}

pub fn create_notifier(token: Token, channel: &Sender<Notify>) -> Notifier {
    Notifier {
        token: token,
        channel: channel.clone()
    }
}

impl Notifier {
    /// Wakeup a state machine
    ///
    ///
    pub fn wakeup(&self) -> Result<(), WakeupError> {
        use mio::NotifyError::*;
        match self.channel.send(Notify::Fsm(self.token)) {
            Ok(()) => Ok(()),
            Err(Closed(_)) => Err(WakeupError::Closed),
            Err(Io(_)) => Err(WakeupError::Io),
            Err(Full(_)) => Err(WakeupError::Full),
        }
    }
}
