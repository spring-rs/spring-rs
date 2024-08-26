use crate::Router;
pub use inventory::submit;

/// TypeHandler is used to configure the spring-macro marked route handler
pub trait TypedHandlerFactory: Send + Sync + 'static {
    /// install route
    fn install_route(&self, router: Router) -> Router;
}

/// Add typed routes marked with procedural macros
pub trait TypeRouter {
    /// Add typed routes marked with procedural macros
    fn typed_route<F: TypedHandlerFactory>(self, factory: F) -> Self;
}

impl TypeRouter for Router {
    fn typed_route<F: TypedHandlerFactory>(self, factory: F) -> Self {
        factory.install_route(self)
    }
}

inventory::collect!(&'static dyn TypedHandlerFactory);

/// auto_config
#[macro_export]
macro_rules! submit_typed_handler {
    ($ty:ident) => {
        ::spring_web::handler::submit! {
            &$ty as &dyn ::spring_web::handler::TypedHandlerFactory
        }
    };
}

/// auto_config
pub fn auto_router() -> Router {
    let mut router = Router::new();
    for handler in inventory::iter::<&dyn TypedHandlerFactory> {
        router = handler.install_route(router);
    }
    router
}
