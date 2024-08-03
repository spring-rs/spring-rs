use anyhow::Context;
use autumn_boot::app::App;
use autumn_macros::{get, route, routes};
use autumn_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin,
};
use autumn_web::{
    error::Result,
    extractor::{Component, Path},
    handler::TypeRouter,
    response::IntoResponse,
    Router, WebConfigurator, WebPlugin,
};

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
        .typed_route(hello_word)
        .typed_route(hello)
        .typed_route(sqlx_request_handler)
}

#[routes]
#[get("/")]
#[get("/hello_world")]
async fn hello_word() -> impl IntoResponse {
    "hello word"
}

#[route("/hello/:name", method = "GET", method = "POST")]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("hello {name}")
}

#[get("/sql")]
async fn sqlx_request_handler(Component(pool): Component<ConnectPool>) -> Result<String> {
    let version = sqlx::query("select version() as version")
        .fetch_one(&pool)
        .await
        .context("sqlx query failed")?
        .get("version");
    Ok(version)
}
