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
}

#[derive(Clone, Service)]
struct UserService {
    #[inject(component)]
    db: ConnectPool,
    #[inject(config)]
    config: UserConfig,
}

impl UserService {
    pub async fn query_db(&self) -> Result<String> {
        let UserConfig { username, project } = &self.config;

        let version: String = sqlx::query("select version() as version")
            .fetch_one(&self.db)
            .await
            .context("sqlx query failed")?
            .get("version");

        Ok(format!(
            "The database used by {username}'s {project} is {version}"
        ))
    }
}

/// For some large-sized components or configs, using Ref can avoid the performance impact of deep copying.
#[derive(Clone, Service)]
struct UserServiceUseRef {
    db: ComponentRef<ConnectPool>,
    config: ConfigRef<UserConfig>,
}

impl UserServiceUseRef {
    pub async fn query_db(&self) -> Result<String> {
        let UserConfig { username, project } = &*self.config;

        let version: String = sqlx::query("select version() as version")
            .fetch_one(&*self.db)
            .await
            .context("sqlx query failed")?
            .get("version");

        Ok(format!(
            "The database used by {username}'s {project} is {version}"
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
