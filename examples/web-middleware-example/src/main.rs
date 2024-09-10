use anyhow::Context;
use spring::App;
use spring_sqlx::sqlx::Row;
use spring_sqlx::{sqlx, ConnectPool, SqlxPlugin};
use spring_web::get;
use spring_web::{
    axum::response::IntoResponse, error::Result, extractor::Component, Router, WebConfigurator,
    WebPlugin,
};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .add_plugin(SqlxPlugin)
        .add_router(router())
        .run()
        .await
}

fn router() -> Router {
    Router::new()
        .merge(spring_web::handler::auto_router())
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
}

#[get("/")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
}

#[get("/version")]
pub async fn sql_version(Component(pool): Component<ConnectPool>) -> Result<String> {
    let version = sqlx::query("select version() as version")
        .fetch_one(&pool)
        .await
        .context("sqlx query failed")?
        .get("version");
    Ok(version)
}
