pub mod config;

use std::time::Duration;

use anyhow::Context;
use async_trait::async_trait;
use autumn_boot::{app::AppBuilder, error::Result, plugin::Plugin};
use config::SeaOrmConfig;
use sea_orm::{ConnectOptions, Database};

pub type DbConn = sea_orm::DbConn;
pub struct SeaOrmPlugin;

#[async_trait]
impl Plugin for SeaOrmPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<SeaOrmConfig>(self)
            .context(format!("sea-orm plugin config load failed"))
            .unwrap();

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
            .sqlx_logging(config.enable_logging);

        if let Some(connect_timeout) = config.connect_timeout {
            opt.connect_timeout(Duration::from_millis(connect_timeout));
        }
        if let Some(idle_timeout) = config.idle_timeout {
            opt.idle_timeout(Duration::from_millis(idle_timeout));
        }
        if let Some(acquire_timeout) = config.acquire_timeout {
            opt.acquire_timeout(Duration::from_millis(acquire_timeout));
        }

        Ok(Database::connect(opt)
            .await
            .with_context(|| format!("sea-orm connection failed:{}", &config.uri))?)
    }
}
