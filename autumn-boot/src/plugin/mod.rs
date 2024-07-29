pub mod component;

use crate::app::AppBuilder;
use async_trait::async_trait;
use std::{any::Any, ops::Deref, sync::Arc};

#[derive(Clone)]
pub struct PluginRef(Arc<dyn Plugin>);

#[async_trait]
pub trait Plugin: Any + Send + Sync {
    /// Configures the [`App`] to which this plugin is added.
    async fn build(&self, app: &mut AppBuilder);

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

impl PluginRef {
    pub(crate) fn new<T: Plugin>(plugin: T) -> Self {
        Self(Arc::new(plugin))
    }
}

impl Deref for PluginRef {
    type Target = dyn Plugin;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
