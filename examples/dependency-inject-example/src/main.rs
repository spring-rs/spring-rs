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

#[derive(Clone, Configurable, Deserialize)]
#[config_prefix = "user"]
struct UserConfig {
    username: String,
    project: String,
}

#[derive(Clone, Service)]
struct UserService {
    #[component]
    db: ConnectPool,
    #[config]
    config: UserConfig,
}

/// For some large-sized components or configs, using Ref can avoid the performance impact of deep copying.
#[derive(Clone, Service)]
struct UserServiceUseRef {
    db: ComponentRef<ConnectPool>,
    config: spring::config::ConfigRef<UserConfig>,
}

#[get("/")]
async fn hello(Component(user_service): Component<UserService>) -> Result<impl IntoResponse> {
    // let a = None;
    // a.expect(format!(""));
    Ok("")
}
