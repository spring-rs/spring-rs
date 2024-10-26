//! This module implements configuration loading.
//!
#![doc = include_str!("../../Config.md")]
/// Environment Configuration
pub mod env;
/// Implement reading toml configuration
pub mod toml;

pub use spring_macros::Configurable;

use crate::error::Result;
use std::{ops::Deref, sync::Arc};

/// The Configurable trait marks whether the struct can read configuration from the [ConfigRegistry]
pub trait Configurable {
    /// Prefix used to read toml configuration.
    /// If you need to load external configuration, you need to rewrite this method
    fn config_prefix() -> &'static str;
}

/// ConfigRegistry is the core trait of configuration management
pub trait ConfigRegistry {
    /// Get the configuration items according to the Configurable's `config_prefix`
    fn get_config<T>(&self) -> Result<T>
    where
        T: serde::de::DeserializeOwned + Configurable;
}

/// ConfigRef avoids cloning of big struct through Arc
#[derive(Debug, Clone)]
pub struct ConfigRef<T: Configurable>(Arc<T>);

impl<T: Configurable> ConfigRef<T> {
    /// constructor
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
        &self.0
    }
}
