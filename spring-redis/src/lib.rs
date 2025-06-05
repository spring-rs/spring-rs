//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io/docs/plugins/spring-redis)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

pub mod config;

pub use redis;
pub use spring_macros::cache;

use anyhow::Context;
use config::RedisConfig;
use redis::{aio::ConnectionManagerConfig, Client};
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::plugin::MutableComponentRegistry;
use spring::{app::AppBuilder, error::Result, plugin::Plugin};
use std::time::Duration;

pub type Redis = redis::aio::ConnectionManager;

pub struct RedisPlugin;

#[async_trait]
impl Plugin for RedisPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<RedisConfig>()
            .expect("redis plugin config load failed");

        let connect: Redis = Self::connect(config).await.expect("redis connect failed");
        app.add_component(connect);
    }
}

impl RedisPlugin {
    async fn connect(config: RedisConfig) -> Result<Redis> {
        let url = config.uri;
        let client = Client::open(url.clone())
            .with_context(|| format!("redis connect failed:{}", url.clone()))?;

        let mut conn_config = ConnectionManagerConfig::new();

        if let Some(exponent_base) = config.exponent_base {
            conn_config = conn_config.set_exponent_base(exponent_base);
        }
        if let Some(factor) = config.factor {
            conn_config = conn_config.set_factor(factor);
        }
        if let Some(number_of_retries) = config.number_of_retries {
            conn_config = conn_config.set_number_of_retries(number_of_retries);
        }
        if let Some(max_delay) = config.max_delay {
            conn_config = conn_config.set_max_delay(max_delay);
        }
        if let Some(response_timeout) = config.response_timeout {
            conn_config = conn_config.set_response_timeout(Duration::from_millis(response_timeout));
        }
        if let Some(connection_timeout) = config.connection_timeout {
            conn_config =
                conn_config.set_connection_timeout(Duration::from_millis(connection_timeout));
        }

        Ok(client
            .get_connection_manager_with_config(conn_config)
            .await
            .with_context(|| format!("redis connect failed:{}", url.clone()))?)
    }
}
