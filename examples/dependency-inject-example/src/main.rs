use anyhow::Context;
use serde::Deserialize;
use spring::{
    auto_config,
    config::{ConfigRef, Configurable},
    plugin::{component::ComponentRef, service::Service},
    App,
};
use spring_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin,
};
use spring_web::get;
use spring_web::{
    axum::response::IntoResponse, error::Result, extractor::Component, WebConfigurator, WebPlugin,
};
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};

// Main function entry
#[auto_config(WebConfigurator)] // auto config web router
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin) // Add plug-in
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[derive(Clone, Configurable, Deserialize)]
#[config_prefix = "user"]
struct UserConfig {
    username: String,
    project: String,
    #[serde(default)]
    init_count: i32,
}

#[derive(Clone, Service)]
struct UserService {
    #[inject(component)]
    db: ConnectPool,
    #[inject(config)]
    config: UserConfig,
    #[inject(func = Self::init_count(&config))]
    count: Arc<AtomicI32>,
}

#[derive(Clone, Service)]
#[prototype]
struct UserProtoService {
    #[inject(component)]
    db: ConnectPool,
    #[inject(config)]
    config: UserConfig,
    #[inject(func = init_zero_count())]
    count: Arc<AtomicI32>,
    // x: i64,
    // y: i32,
}

impl UserService {
    pub async fn query_db(&self) -> Result<String> {
        let UserConfig {
            username, project, ..
        } = &self.config;

        let version: String = sqlx::query("select version() as version")
            .fetch_one(&self.db)
            .await
            .context("sqlx query failed")?
            .get("version");

        let pv_count = self.count.fetch_add(1, Ordering::SeqCst);
        Ok(format!(
            r#"
            The database used by {username}'s {project} is {version}.
            Page view counter is {pv_count}
            "#
        ))
    }

    fn init_count(config: &UserConfig) -> Arc<AtomicI32> {
        Arc::new(AtomicI32::new(config.init_count))
    }
}

/// For some large-sized components or configs, using Ref can avoid the performance impact of deep copying.
#[derive(Clone, Service)]
struct UserServiceUseRef {
    db: ComponentRef<ConnectPool>,
    config: ConfigRef<UserConfig>,
    #[inject(func = init_zero_count())]
    count: Arc<AtomicI32>,
}

fn init_zero_count() -> Arc<AtomicI32> {
    Arc::new(AtomicI32::new(0))
}

impl UserServiceUseRef {
    pub async fn query_db(&self) -> Result<String> {
        let UserConfig {
            username, project, ..
        } = &*self.config;

        let version: String = sqlx::query("select version() as version")
            .fetch_one(&*self.db)
            .await
            .context("sqlx query failed")?
            .get("version");

        let pv_count = self.count.fetch_add(1, Ordering::SeqCst);
        Ok(format!(
            r#"
            The database used by {username}'s {project} is {version}.
            Page view counter is {pv_count}
            "#
        ))
    }
}

#[get("/")]
async fn hello(Component(user_service): Component<UserService>) -> Result<impl IntoResponse> {
    Ok(user_service.query_db().await?)
}

#[get("/use-ref")]
async fn hello_ref(
    Component(user_service): Component<UserServiceUseRef>,
) -> Result<impl IntoResponse> {
    Ok(user_service.query_db().await?)
}
