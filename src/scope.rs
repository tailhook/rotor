use mio::Token;
use std::ops::{Deref, DerefMut};


pub struct Scope<'a, C:Sized+'a>(Token, &'a mut C);


impl<'a, C> Deref for Scope<'a, C> {
    type Target = C;
    fn deref(&self) -> &C {
        self.1
    }
}

impl<'a, C> DerefMut for Scope<'a, C> {
    fn deref_mut(&mut self) -> &mut C {
        self.1
    }
}

pub fn scope<C>(token: Token, ctx: &mut C) -> Scope<C> {
    Scope(token, ctx)
}
