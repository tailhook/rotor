use {Response};


impl<M: Sized> Response<M> {
    pub fn ok(machine: M) -> Response<M> {
        Response(Some(machine), None)
    }
    pub fn spawn(machine: M, result: M) -> Response<M> {
        Response(Some(machine), Some(result))
    }
    pub fn done() -> Response<M> {
        Response::<M>(None, None)
    }
    pub fn map<T, S, R>(self, self_mapper: S, result_mapper: R) -> Response<T>
        where S: FnOnce(M) -> T,
              R: FnOnce(M) -> T
    {
        Response(self.0.map(self_mapper), self.1.map(result_mapper))
    }
}

