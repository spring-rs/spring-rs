use anyhow::Context;
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
    axum::response::IntoResponse,
    error::Result,
    extractor::{Component, Path},
    WebConfigurator, WebPlugin,
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

#[derive(Clone, Configurable)]
#[config_prefix = "user"]
struct UserConfig {
    username: String,
    project: String,
}

#[derive(Clone, Service)]
struct UserService {
    db: ComponentRef<ConnectPool>,
    config: ConfigRef<UserConfig>,
}

#[get("/")]
async fn hello(Component(user_service): Component<UserService>) -> Result<impl IntoResponse> {
    Ok("")
}
