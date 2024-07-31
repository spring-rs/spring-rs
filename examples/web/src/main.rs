use anyhow::Context;
use autumn_boot::app::App;
use autumn_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin,
};
use autumn_web::{error::Result, extractor::Component, get, Router, WebConfigurator, WebPlugin};

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .add_router(router())
        .run()
        .await
}

fn router() -> Router {
    Router::new()
        .route("/", get(hello_word))
        .route("/sql", get(sqlx_request_handler))
}

async fn hello_word() -> &'static str {
    "hello word"
}

async fn sqlx_request_handler(Component(pool): Component<ConnectPool>) -> Result<String> {
    let version = sqlx::query("select version() as version")
        .fetch_one(&pool)
        .await
        .context("sqlx query failed")?
        .get("version");
    Ok(version)
}
