use crate::app::AppBuilder;
use crate::error::Result;
pub use spring_macros::Service;

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
pub trait Service: Sized {
    fn build(app: &AppBuilder) -> Result<Self>;
}
