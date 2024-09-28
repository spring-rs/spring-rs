//! [spring](https://spring-rs.github.io/docs/plugins/plugin-by-self/)
#![doc = include_str!("../README.md")]
#![doc = include_str!("../DI.md")]

/// App Builder
pub mod app;
/// App Config
pub mod config;
pub mod error;
pub mod log;
pub mod plugin;

pub use app::App;
pub use async_trait::async_trait;
pub use spring_macros::auto_config;
pub use tracing;
