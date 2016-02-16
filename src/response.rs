use std::error::Error;

use mio::Token;

use {Response, Time};


#[derive(Debug)]
pub enum ResponseImpl<M, N> {
    Normal(M),
    Deadline(M, Time),
    Spawn(M, N),
    Error(Box<Error>),
    Done,
}

impl<M: Sized, N:Sized> Response<M, N> {
    pub fn ok(machine: M) -> Response<M, N> {
        Response(ResponseImpl::Normal(machine))
    }
    pub fn spawn(machine: M, result: N) -> Response<M, N> {
        Response(ResponseImpl::Spawn(machine, result))
    }
    pub fn done() -> Response<M, N> {
        Response::<M, N>(ResponseImpl::Done)
    }
    pub fn error(e: Box<Error>) -> Response<M, N> {
        Response::<M, N>(ResponseImpl::Error(e))
    }
    pub fn deadline(self, time: Time) -> Response<M, N> {
        let imp = match self.0 {
            ResponseImpl::Normal(x) => ResponseImpl::Deadline(x, time),
            ResponseImpl::Deadline(x, _) => ResponseImpl::Deadline(x, time),
            ResponseImpl::Spawn(..) => {
                panic!("You can't attach a deadline/timeout to the \
                    Response::spawn(). The `spawn` action is synchronous \
                    you must set a deadline in the `spawned` handler."); }
            ResponseImpl::Done => {
                panic!("You can't attach a deadline/timeout to \
                    Response::done() as it's useless. \
                    Timeout will never happen");
            }
            ResponseImpl::Error(_) => {
                panic!("You can't attach a deadline/timeout to \
                    Response::error(_) as it's useless. \
                    Timeout will never happen");
            }
        };
        Response(imp)
    }
    /// Maps state machine and/or spawned result with a function
    ///
    /// Usually it's okay to use constructor of wrapper state machine
    /// here as a mapper
    pub fn map<T, U,  S, R>(self, self_mapper: S, result_mapper: R)
        -> Response<T, U>
        where S: FnOnce(M) -> T,
              R: FnOnce(N) -> U,
    {
        use self::ResponseImpl::*;
        let imp = match self.0 {
            Normal(m) => Normal(self_mapper(m)),
            Deadline(m, time) => Deadline(self_mapper(m), time),
            Spawn(m, n) => Spawn(self_mapper(m), result_mapper(n)),
            Done => Done,
            Error(e) => Error(e),
        };
        Response(imp)
    }
    /// Similar to `map` but only maps state machine
    ///
    /// This is especially useful in state machine constructors, which
    /// have a Void child type.
    pub fn wrap<T, S>(self, self_mapper: S) -> Response<T, N>
        where S: FnOnce(M) -> T
    {
        use self::ResponseImpl::*;
        let imp = match self.0 {
            Normal(m) => Normal(self_mapper(m)),
            Deadline(m, time) => Deadline(self_mapper(m), time),
            Spawn(m, n) => Spawn(self_mapper(m), n),
            Done => Done,
            Error(e) => Error(e),
        };
        Response(imp)
    }
}

pub fn decompose<M, N>(token: Token, res: Response<M, N>)
    -> (Result<M, Option<Box<Error>>>, Option<N>, Option<Time>)
{
    match res.0 {
        ResponseImpl::Normal(m) => (Ok(m), None, None),
        ResponseImpl::Deadline(m, time) => (Ok(m), None, Some(time)),
        ResponseImpl::Spawn(m, n) => (Ok(m), Some(n), None),
        ResponseImpl::Done => (Err(None), None, None),
        ResponseImpl::Error(e) => {
            if cfg!(log_errors) {
                warn!("State machine {:?} exited with error: {}", token, e);
            }
            (Err(Some(e)), None, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Response;

    #[test]
    fn size_of_response() {
        assert_eq!(::std::mem::size_of::<Response<u64, u64>>(), 24)
    }
}
