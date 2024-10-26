#![doc = include_str!("../../README.md")]

/// Component definition
pub mod component;
pub mod service;

use crate::app::AppBuilder;
use async_trait::async_trait;
use std::{any::Any, ops::Deref, sync::Arc};

/// Plugin Reference
#[derive(Clone)]
pub struct PluginRef(Arc<dyn Plugin>);

/// Defined plugin interface
#[async_trait]
pub trait Plugin: Any + Send + Sync {
    /// Configures the `App` to which this plugin is added.
    async fn build(&self, _app: &mut AppBuilder) {}

    /// Configures the `App` to which this plugin is added.
    /// The immediately plugin will not be added to the registry, 
    /// and the plugin cannot obtain components registered in the registry.
    fn immediately_build(&self, _app: &mut AppBuilder) {}

    /// Configures a name for the [`Plugin`] which is primarily used for checking plugin
    /// uniqueness and debugging.
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// A list of plugins to depend on. The plugin will be built after the plugins in this list.
    fn dependencies(&self) -> Vec<&str> {
        vec![]
    }

    /// Whether the plugin should be built immediately when added
    fn immediately(&self) -> bool {
        false
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
