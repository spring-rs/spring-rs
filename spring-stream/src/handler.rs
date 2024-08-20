use std::{future::Future, pin::Pin};

pub trait Handler<T>: Clone + Send + Sized + 'static {
    /// The type of future calling this handler returns.
    type Future: Future<Output = ()> + Send + 'static;

    /// Call the handler with the given request.
    fn call(self) -> Self::Future;
}

/// no args handler impl
impl<F, Fut> Handler<()> for F
where
    F: FnOnce() -> Fut + Clone + Send + 'static,
    Fut: Future<Output = ()> + Send,
{
    type Future = Pin<Box<dyn Future<Output = ()> + Send>>;

    fn call(self) -> Self::Future {
        Box::pin(async move {
            self().await;
        })
    }
}

#[derive(Clone)]
pub struct BoxedHandler {}

impl BoxedHandler {
    pub(crate) fn from_handler<H, T>(handler: H) -> Self
    where
        H: Handler<T> + Sync,
        T: 'static,
    {
        // TODO
        Self {}
    }
}
