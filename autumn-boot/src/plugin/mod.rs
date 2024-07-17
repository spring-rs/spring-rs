use crate::app::App;
use async_trait::async_trait;
use std::any::Any;

#[async_trait]
pub trait Plugin: Any + Send + Sync {
    /// Configures the [`App`] to which this plugin is added.
    async fn build(&self, app: &mut App);

    /// Prefix used to read toml configuration.
    /// If you need to load external configuration, you need to rewrite this method
    fn config_prefix(&self) -> &str;

    /// Configures a name for the [`Plugin`] which is primarily used for checking plugin
    /// uniqueness and debugging.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// A list of plugins to depend on. The plugin will be built after the plugins in this list.
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }
}
