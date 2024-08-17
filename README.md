<div align="center">
    <img src="docs/static/logo.svg" alt="Logo"/>
    <h3>spring-rs is microservice framework written in rust</h3>
    <div>English ｜ <a href="./README.zh.md">中文</a></div>
</div>

<b>spring-rs</b> is a microservice framework written in rust, similar to springboot in java. <b>spring-rs</b> provides an easily extensible plug-in system for integrating excellent projects in the rust community, such as axum, sqlx, sea-orm, etc.

Compared with springboot in java, spring-rs has higher performance and lower memory usage, allowing you to completely get rid of the bloated JVM and travel light.

## Example

```rust
use spring::{nest, route, routes, auto_config, App};
use spring_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin
};
use spring_web::{
    extractor::Path, handler::TypeRouter, response::IntoResponse, Router, WebConfigurator,
    WebPlugin,
};

#[tokio::main]
#[auto_config(WebConfigurator)]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[get("/")]
async fn hello_word() -> impl IntoResponse {
    "hello word"
}

#[route("/hello/:name", method = "GET", method = "POST")]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("hello {name}")
}

#[get("/version")]
async fn sqlx_request_handler(Component(pool): Component<ConnectPool>) -> Result<String> {
    let version = sqlx::query("select version() as version")
        .fetch_one(&pool)
        .await
        .context("sqlx query failed")?
        .get("version");
    Ok(version)
}
```
