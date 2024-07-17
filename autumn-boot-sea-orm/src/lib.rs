pub mod config;

use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use autumn_boot::{error::Result, plugin::Plugin};
use config::SeaOrmConfig;
use sea_orm::{ConnectOptions, Database};

pub type DbConn = sea_orm::DbConn;
pub struct SeaOrmPlugin;

#[async_trait]
impl Plugin for SeaOrmPlugin {
    async fn build(&self, app: &mut autumn_boot::app::App) {
        let config = app
            .get_config::<SeaOrmConfig>(self)
            .context(format!("sqlx plugin config load failed"))
            .expect("sea-orm plugin load failed");

        let conn = Self::connect(&config)
            .await
            .expect("sea-orm plugin load failed");
        app.add_component(conn);
    }

    fn config_prefix(&self) -> &str {
        "sea-orm"
    }
}

impl SeaOrmPlugin {
    pub async fn connect(config: &config::SeaOrmConfig) -> Result<DbConn> {
        let mut opt = ConnectOptions::new(&config.uri);
        opt.max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .connect_timeout(Duration::from_millis(config.connect_timeout))
            .idle_timeout(Duration::from_millis(config.idle_timeout))
            .sqlx_logging(config.enable_logging);

        if let Some(acquire_timeout) = config.acquire_timeout {
            opt.acquire_timeout(Duration::from_millis(acquire_timeout));
        }

        Ok(Database::connect(opt)
            .await
            .with_context(|| format!("sea-orm connection failed:{}", &config.uri))?)
    }
}
