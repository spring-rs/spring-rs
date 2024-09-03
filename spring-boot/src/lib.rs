//! spring-boot
#[cfg(not(doctest))]
#[doc = include_str!("../README.md")]
/// App Builder
pub mod app;
/// App Config
pub mod config;
pub mod error;
pub mod log;
pub mod plugin;
pub use async_trait::async_trait;
pub use tracing;