#![doc = include_str!("../../Plugin.md")]

/// Component definition
pub mod component;
/// Lazy component loading for circular dependencies
pub mod lazy;
/// Service is a special Component that supports dependency injection at compile time
pub mod service;

use crate::error::Result;
use crate::{app::AppBuilder, error::AppError};
use async_trait::async_trait;
use component::ComponentRef;
pub use lazy::LazyComponent;
use std::{
    any::{self, Any},
    ops::Deref,
    sync::Arc,
};
pub use service::Service;

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

/// Component Registry
pub trait ComponentRegistry {
    /// Get the component reference of the specified type
    fn get_component_ref<T>(&self) -> Option<ComponentRef<T>>
    where
        T: Any + Send + Sync;

    /// Get the component reference of the specified type.
    /// If the component does not exist, it will panic.
    fn get_expect_component_ref<T>(&self) -> ComponentRef<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.get_component_ref().unwrap_or_else(|| {
            panic!(
                "{} component not exists in registry",
                std::any::type_name::<T>()
            )
        })
    }

    /// Get the component reference of the specified type.
    /// If the component does not exist, it will return AppError::ComponentNotExist.
    fn try_get_component_ref<T>(&self) -> Result<ComponentRef<T>>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.get_component_ref()
            .ok_or_else(|| AppError::ComponentNotExist(std::any::type_name::<T>()))
    }

    /// Get the component of the specified type
    fn get_component<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static;

    /// Get the component of the specified type.
    /// If the component does not exist, it will panic.
    fn get_expect_component<T>(&self) -> T
    where
        T: Clone + Send + Sync + 'static,
    {
        self.get_component().unwrap_or_else(|| {
            panic!(
                "{} component not exists in registry",
                std::any::type_name::<T>()
            )
        })
    }

    /// Get the component of the specified type.
    /// If the component does not exist, it will return AppError::ComponentNotExist.
    fn try_get_component<T>(&self) -> Result<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.get_component()
            .ok_or_else(|| AppError::ComponentNotExist(std::any::type_name::<T>()))
    }

    /// Is there a component of the specified type in the registry?
    fn has_component<T>(&self) -> bool
    where
        T: Any + Send + Sync;
}

/// Mutable Component Registry
pub trait MutableComponentRegistry: ComponentRegistry {
    /// Add component to the registry
    fn add_component<C>(&mut self, component: C) -> &mut Self
    where
        C: Clone + any::Any + Send + Sync;
}
