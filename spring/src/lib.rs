#![deny(missing_docs)]
//! For the complete documentation of spring, please click this address: [https://spring-rs.github.io]
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]
#![doc = include_str!("../../README.md")]

/// App Builder
pub mod app;
/// Config System: 
pub mod config;
/// spring-rs definition error
pub mod error;
/// The log plugin is a built-in plugin of spring-rs and is also the first plugin loaded when the application starts.
pub mod log;
/// Plugin system: Through the documentation of this module you will learn how to implement your own plugins
pub mod plugin;
/// Banner
pub mod banner;

pub use app::App;
pub use async_trait::async_trait;
pub use spring_macros::auto_config;
pub use tracing;
pub use tracing_error::SpanTrace;

pub use spring_macros::main;