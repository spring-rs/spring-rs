use std::future::Future;

pub trait Handler<ARGS>: Clone + Send + Sized + 'static {
    /// The type of future calling this handler returns.
    type Future: Future<Output = ()> + Send + 'static;

    /// Call the handler with the given request.
    fn call(self, args: ARGS) -> Self::Future;
}
