pub mod config;

use anyhow::Context;
use async_trait::async_trait;
use autumn_boot::{app::AppBuilder, error::Result, plugin::Plugin};
use config::RedisConfig;
use redis::Client;

pub type Redis = redis::aio::ConnectionManager;

pub struct RedisPlugin;

#[async_trait]
impl Plugin for RedisPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<RedisConfig>(self)
            .context(format!("redis plugin config load failed"))
            .expect("redis plugin load failed");

        let connect: Redis = Self::connect(config).await.expect("redis connect failed");
        app.add_component(connect);
    }

    fn config_prefix(&self) -> &str {
        "redis"
    }
}

impl RedisPlugin {
    async fn connect(config: RedisConfig) -> Result<Redis> {
        let url = config.uri;
        let client = Client::open(url.clone())
            .with_context(|| format!("redis connect failed:{}", url.clone()))?;
        Ok(client
            .get_connection_manager()
            .await
            .with_context(|| format!("redis connect failed:{}", url.clone()))?)
    }
}
