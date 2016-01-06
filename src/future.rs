use {Notify, Port, Future};

impl<T:Sized> Port<T> {
    /// Set the value of the future
    ///
    /// Returns `Err(value)` if the future in the peer is already dropped.
    ///
    /// # Panics
    ///
    /// Panics when message to the target main loop can't be sent
    pub fn set(self, value: T) -> Result<(), T> {
        if let Some(arc) = self.contents.upgrade() {
            *arc.lock().expect("Lock of the future is poisoned") = Some(value);
            self.channel.send(Notify::Fsm(self.token))
                .expect("Target channel for the future is full");
            Ok(())
        } else {
            Err(value)
        }
    }
}

impl<T:Sized> Future<T> {
    /// Get the value consuming the future
    ///
    /// # Panics
    ///
    /// Panics when no value is set yet
    pub fn get(self) -> T {
        self.contents.lock().expect("Lock of the future is poisoned")
        .take().expect("Future is not resolved yet")
    }
    pub fn take(self) -> Result<T, Future<T>> {
        let arc = self.contents;
        let val = arc.lock().expect("Lock of the future is poisoned")
                     .take();
        val.ok_or(Future { contents: arc })
    }
    /// Check if there is a value in the future
    pub fn is_done(&self) -> bool {
        self.contents.lock()
            .expect("Lock of the future is poisoned").is_some()
    }
}
