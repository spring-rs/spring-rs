//! [spring-postgres](https://spring-rs.github.io/docs/plugins/spring-postgres/)
pub mod config;
pub extern crate tokio_postgres as postgres;

use config::PgConfig;
use spring_boot::app::AppBuilder;
use spring_boot::async_trait;
use spring_boot::config::Configurable;
use spring_boot::plugin::Plugin;
use std::ops::Deref;
use std::sync::Arc;
use tokio_postgres::NoTls;

#[derive(Clone)]
pub struct Postgres(Arc<tokio_postgres::Client>);

impl Postgres {
    fn new(client: tokio_postgres::Client) -> Self {
        Self(Arc::new(client))
    }
}

impl Deref for Postgres {
    type Target = tokio_postgres::Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Configurable)]
#[config_prefix = "postgres"]
pub struct PgPlugin;

#[async_trait]
impl Plugin for PgPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<PgConfig>(self)
            .expect("postgres plugin config load failed");

        let (client, connection) = tokio_postgres::connect(&config.connect, NoTls)
            .await
            .expect("connect postgresql failed");

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                tracing::error!("postgresql connection error: {}", e);
            }
        });

        app.add_component(Postgres::new(client));
    }
}
