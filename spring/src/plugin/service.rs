use crate::app::AppBuilder;
use crate::error::Result;
pub use spring_macros::Service;
pub use inventory::submit;

/// Service is a special Component that can inject dependent Components as field members
/// ```rust
/// #[derived(Service)]
/// struct UserService {
///     #[component]
///     db: DbConn,
///     redis: Redis,
///     #[config]
///     config: CustomConfig,
/// }
/// ```
pub trait Service: Clone + Sized {
    fn build(app: &AppBuilder) -> Result<Self>;
}

pub trait ServiceRegistrar: Send + Sync + 'static {
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

pub fn auto_inject_service(app: &mut AppBuilder) -> Result<()> {
    for registrar in inventory::iter::<&dyn ServiceRegistrar> {
        registrar.install_service(app)?;
    }
    Ok(())
}
