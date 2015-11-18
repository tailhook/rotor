use std::ops::{Deref, DerefMut};

use mio::{Token, Sender};

use handler::Notify;
use future::{pair, Future, Port};


pub struct Scope<'a, C:Sized+'a>{
    token: Token,
    ctx: &'a mut C,
    channel: &'a mut Sender<Notify>,
}


impl<'a, C> Deref for Scope<'a, C> {
    type Target = C;
    fn deref(&self) -> &C {
        self.ctx
    }
}

impl<'a, C> DerefMut for Scope<'a, C> {
    fn deref_mut(&mut self) -> &mut C {
        self.ctx
    }
}

pub fn scope<'x, C>(token: Token, ctx: &'x mut C,
    channel: &'x mut Sender<Notify>)
    -> Scope<'x, C>
{
    Scope {
        token: token,
        ctx: ctx,
        channel: channel,
    }
}

impl<'a, C> Scope<'a, C> {
    pub fn create_future<T:Sized>(&mut self) -> (Port<T>, Future<T>) {
        pair(self.token, self.channel)
    }
}
