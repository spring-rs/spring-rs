pub mod env;
pub mod toml;

use std::{ops::Deref, sync::Arc};

pub use spring_macros::Configurable;

use crate::error::Result;

pub trait Configurable {
    /// Prefix used to read toml configuration.
    /// If you need to load external configuration, you need to rewrite this method
    fn config_prefix() -> &'static str;
}

pub trait ConfigRegistry {
    /// Get the configuration items according to the Configurable's `config_prefix`
    fn get_config<T>(&self) -> Result<T>
    where
        T: serde::de::DeserializeOwned + Configurable;
}

#[derive(Debug, Clone)]
pub struct ConfigRef<T: Configurable>(Arc<T>);

impl<T: Configurable> ConfigRef<T> {
    pub fn new(config: T) -> Self {
        Self(Arc::new(config))
    }
}

impl<T> Deref for ConfigRef<T>
where
    T: Configurable,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
