//! spring-boot
#[doc = include_str!("../README.md")]
pub mod app;
pub mod config;
pub mod error;
pub mod log;
pub mod plugin;
pub use async_trait::async_trait;
pub use tracing;