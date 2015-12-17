use {Response};


impl<M: Sized, N:Sized> Response<M, N> {
    pub fn ok(machine: M) -> Response<M, N> {
        Response(Some(machine), None)
    }
    pub fn spawn(machine: M, result: N) -> Response<M, N> {
        Response(Some(machine), Some(result))
    }
    pub fn done() -> Response<M, N> {
        Response::<M, N>(None, None)
    }
    pub fn map<T, U,  S, R>(self, self_mapper: S, result_mapper: R)
        -> Response<T, U>
        where S: FnOnce(M) -> T,
              R: FnOnce(N) -> U,
    {
        Response(self.0.map(self_mapper), self.1.map(result_mapper))
    }
}

