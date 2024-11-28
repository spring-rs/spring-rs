//! Service is a special Component that supports dependency injection at compile time
#![doc = include_str!("../../DI.md")]
use crate::app::AppBuilder;
use crate::error::Result;

pub use inventory::submit;
pub use spring_macros::Service;

use super::ComponentRegistry;

/// Service is a special Component that can inject dependent Components as field members
/// ```rust
/// use spring::plugin::service::Service;
/// use spring_sqlx::ConnectPool;
///
/// #[derive(Clone, Service)]
/// struct UserService {
///     #[component]
///     db: ConnectPool
/// }
/// ```
pub trait Service: Clone + Sized {
    /// Construct the Service component
    fn build<R: ComponentRegistry>(registry: &R) -> Result<Self>;

    /// Whether the service is a prototype service.
    /// If it is a prototype service, each call to ComponentRegistry::get_component will rebuild a new Service object.
    fn prototype() -> bool {
        false
    }
}

/// Install the Service component into the App
pub trait ServiceRegistrar: Send + Sync + 'static {
    /// Install the Service component into the App
    fn install_service(&self, app: &mut AppBuilder) -> Result<()>;
}

inventory::collect!(&'static dyn ServiceRegistrar);

/// auto_config
#[macro_export]
macro_rules! submit_service {
    ($ty:ident) => {
        ::spring::plugin::service::submit! {
            &$ty as &dyn ::spring::plugin::service::ServiceRegistrar
        }
    };
}

/// Find all ServiceRegistrar and install them into the app
pub fn auto_inject_service(app: &mut AppBuilder) -> Result<()> {
    for registrar in inventory::iter::<&dyn ServiceRegistrar> {
        registrar.install_service(app)?;
    }
    Ok(())
}
