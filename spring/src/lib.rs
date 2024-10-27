#![deny(missing_docs)]
//! For the complete documentation of spring, please click this address: [https://spring-rs.github.io]
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]
//! ## Supported plugins
//! * [x] ![spring-web](https://img.shields.io/crates/v/spring-web.svg)[`spring-web`](/spring-web)(Based on [`axum`](https://github.com/tokio-rs/axum))
//! * [x] ![spring-sqlx](https://img.shields.io/crates/v/spring-sqlx.svg)[`spring-sqlx`](/spring-sqlx)(Integrated with [`sqlx`](https://github.com/launchbadge/sqlx))
//! * [x] ![spring-postgres](https://img.shields.io/crates/v/spring-postgres.svg)[`spring-postgres`](/spring-postgres)(Integrated with [`rust-postgres`](https://github.com/sfackler/rust-postgres))
//! * [x] ![spring-sea-orm](https://img.shields.io/crates/v/spring-sea-orm.svg)[`spring-sea-orm`](/spring-sea-orm)(Integrated with [`sea-orm`](https://www.sea-ql.org/SeaORM/))
//! * [x] ![spring-redis](https://img.shields.io/crates/v/spring-redis.svg)[`spring-redis`](/spring-redis)(Integrated with [`redis`](https://github.com/redis-rs/redis-rs))
//! * [x] ![spring-mail](https://img.shields.io/crates/v/spring-mail.svg)[`spring-mail`](/spring-mail)(integrated with [`lettre`](https://github.com/lettre/lettre))
//! * [x] ![spring-job](https://img.shields.io/crates/v/spring-job.svg)[`spring-job`](/spring-job)(integrated with [`tokio-cron-scheduler`](https://github.com/mvniekerk/tokio-cron-scheduler))
//! * [x] ![spring-stream](https://img.shields.io/crates/v/spring-stream.svg)[`spring-stream`](/spring-stream)(Integrate [`sea-streamer`](https://github.com/SeaQL/sea-streamer) to implement message processing such as redis-stream and kafka)
//! * [x] ![spring-opentelemetry](https://img.shields.io/crates/v/spring-opentelemetry.svg)[`spring-opentelemetry`](/spring-opentelemetry)(integrate with [`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust) to implement full observability of logging, metrics, tracing)
//! * [ ] `spring-tarpc`(Integrate[`tarpc`](https://github.com/google/tarpc) to implement RPC calls)
//! 
//! ## Ecosystem
//! 
//! * ![spring-sqlx-migration-plugin](https://img.shields.io/crates/v/spring-sqlx-migration-plugin.svg) [`spring-sqlx-migration-plugin`](https://github.com/Phosphorus-M/spring-sqlx-migration-plugin)

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

pub use app::App;
pub use async_trait::async_trait;
pub use spring_macros::auto_config;
pub use tracing;
