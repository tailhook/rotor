use async::Async;

pub trait Machine: Sized {
    type Message: Send;
    type Value: Sized;
    fn message(self, msg: Self::Message) -> Async<Self, Self::Value>;
}
