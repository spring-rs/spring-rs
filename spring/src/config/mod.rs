pub mod env;
pub mod toml;

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
