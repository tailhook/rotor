use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};
use std::ops::{Deref, DerefMut};

use mio::{Token, Sender};

use scope;
use {Scope, Notify};

pub struct Guard<'a, T:'a>(PeerN, MutexGuard<'a, CondInternal<T>>);
pub struct Peer1Monitor<T>(Arc<Mutex<CondInternal<T>>>);
pub struct Peer2Monitor<T>(Arc<Mutex<CondInternal<T>>>);
pub struct Peer2Token<T>(Option<Arc<Mutex<CondInternal<T>>>>);
#[derive(Debug)]
pub struct NoPeer;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum PeerN {
    First,
    Second,
}

/// A primitive for full-duplex communication between two state machines
///
/// The primitive allows to wakeup both peers and shares value T between
/// them. But it doesn't declare what T is. It can be queue or a counter or
/// any other value.
///
/// The transparency on T allows to put (part of) connection state machine
/// A here and therefore write data directly from the state machine B. This
/// allows to switch between state machines only when TCP pushback is applied.
pub trait Monitor<T> {
    fn consume<'x>(&'x self) -> Option<Guard<'x, T>>;
}

impl<'a, T> Guard<'a, T> {
    /// Send notification to other peer
    ///
    /// Returns `true` if peer will wake up, regardless whether it was already
    /// scheduled for wakeup. The return value of `false` means that peer is
    /// either not yet connected or already closed (shut down).
    ///
    /// # Panics
    ///
    /// When notify queue in mio is full the method panics.
    fn notify_peer(mut self) -> Result<bool, Guard<'a, T>> {
        {
            let peer = match self.0 {
                // Notify the opposite peer
                PeerN::First => &mut self.1.peer2,
                PeerN::Second => &mut self.1.peer1,
            };
            match peer {
                &mut Peer::Operating { token, ref channel, ref mut pending }
                => {
                    if !*pending {
                        *pending = true;
                        channel.send(Notify::Fsm(token))
                               .expect("Can't send to notify queue");
                        return Ok(true);
                    } else {
                        return Ok(false);
                    }
                }
                _ => {}
            }
        }
        Err(self)
    }
}

enum Peer {
    /// Internal already created and other peer is not known yet
    Connecting,
    /// Peer ready to accept messages and wake-ups
    ///
    /// `pending` flag is `true` when message for wake-up is already
    /// queued but not yet consumed
    Operating { pending: bool, token: Token, channel: Sender<Notify> },
    /// Peer state machine is already shutdown or is unable to receive
    /// messages any more. In other words will never wake up from this
    /// condition variable.
    Closed,
}

struct CondInternal<T> {
    peer1: Peer,
    peer2: Peer,
    data: T,
}

impl<T> Monitor<T> {
    pub fn create<C:Sized>(initial_value: T, scope: &Scope<C>)
        -> (Peer1Monitor<T>, Peer2Token<T>)
    {
        let intern = Arc::new(Mutex::new(CondInternal {
            peer1: Peer::Operating {
                token: scope::get_token(scope),
                channel: scope::get_channel(scope),
                pending: false,
                },
            peer2: Peer::Connecting,
            data: initial_value,
        }));
        (Peer1Monitor(intern.clone()), Peer2Token(Some(intern)))
    }
}


impl<T> Peer2Token<T> {
    /// Creates a peer's monitor structure consuming token
    ///
    /// # Panics
    ///
    /// When underlying mutex is poisoned
    // TODO(tailhook) better error?
    //     should notify_peer errors be exposed here insted of panic?
    pub fn connect<C:Sized>(mut self, scope: &Scope<C>)
        -> Result<Peer2Monitor<T>, NoPeer>
    {
        let arc = self.0.take().unwrap();
        {
            let mut guard = Guard(PeerN::Second,
                arc.lock().expect("monitor lock is poisoned"));
            guard.1.peer2 = Peer::Operating {
                token: scope::get_token(scope),
                channel: scope::get_channel(scope),
                pending: false,
                };
            if !guard.notify_peer().is_ok() {
                return Err(NoPeer);
            }
        }
        Ok(Peer2Monitor(arc))
    }
}

impl<T> Monitor<T> for Peer1Monitor<T> {
    fn consume<'x>(&'x self) -> Option<Guard<'x, T>> {
        let mut guard = self.0.lock()
            .expect("monitor lock is poisoned");
        match guard.peer1 {
            Peer::Connecting => unreachable!(),
            Peer::Operating { ref mut pending, .. } => {
                if !*pending {
                    return None;
                }
                *pending = false;
            }
            Peer::Closed => unreachable!(),
        }
        Some(Guard(PeerN::First, guard))
    }
}
impl<T> Drop for Peer1Monitor<T> {
    fn drop(&mut self) {
        self.0.lock().map(|mut x| {
            x.deref_mut().peer1 = Peer::Closed;
            Guard(PeerN::First, x).notify_peer()
        }).ok();
    }
}

impl<T> Monitor<T> for Peer2Monitor<T> {
    fn consume<'x>(&'x self) -> Option<Guard<'x, T>> {
        let mut guard = self.0.lock()
            .expect("monitor lock is poisoned");
        match guard.peer2 {
            Peer::Connecting => unreachable!(),
            Peer::Operating { ref mut pending, .. } => {
                if !*pending {
                    return None
                }
                *pending = false;
            }
            Peer::Closed => unreachable!(),
        }
        Some(Guard(PeerN::Second, guard))
    }
}
impl<T> Drop for Peer2Monitor<T> {
    fn drop(&mut self) {
        self.0.lock().map(|mut x| {
            x.deref_mut().peer2 = Peer::Closed;
            Guard(PeerN::Second, x).notify_peer()
        }).ok();
    }
}
impl<T> Drop for Peer2Token<T> {
    fn drop(&mut self) {
        if let Some(ref arc) = self.0 {
            arc.lock().map(|mut x| {
                x.deref_mut().peer2 = Peer::Closed;
                Guard(PeerN::Second, x).notify_peer()
            }).ok();
        }
    }
}

impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.1.data
    }
}
impl<'a, T> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.1.data
    }
}

impl<'a, T> fmt::Debug for Guard<'a, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Guard({:?})", self.0)
    }
}

#[cfg(test)]
mod test {
    use super::Monitor;
    use test_util::Loop;

    #[test]
    fn ping_pong() {
        let mut counter = 0;
        {
            let mut lp = Loop::new(());
            let (mon1, tok) = Monitor::create(&mut counter, &lp.scope(1));
            let mon2 = tok.connect(&lp.scope(2)).unwrap();
            {
                let mut guard = mon1.consume().unwrap();
                **guard = 3;
                guard.notify_peer().unwrap();
            }
            assert!(mon1.consume().is_none());
            {
                let mut guard = mon2.consume().unwrap();
                **guard *= 2;
                guard.notify_peer().unwrap();
            }
            assert!(mon1.consume().is_some());
            assert!(mon2.consume().is_none());
            assert!(mon1.consume().is_none());
        }
        assert_eq!(counter, 6);
    }
}

