trait Handler {
    type Timeout: Sized;
    fn timeout(self, timeout: Self::Timeout) -> Option<Self>;
}
