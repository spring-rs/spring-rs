<div align="center">
    <img src="docs/static/logo.svg" alt="Logo"/>
    <h3>spring-rs is microservice framework written in Rust</h3>
    <p>English ｜ <a href="./README.zh.md">中文</a></p>
    <p>
        <a href="https://crates.io/crates/spring">
            <img src="https://img.shields.io/crates/v/spring.svg" alt="crates.io"/>
        </a>
        <a href="https://docs.rs/spring">
            <img src="https://docs.rs/spring/badge.svg" alt="Documentation"/>
        </a>
    </p>
</div>

<b>spring-rs</b> is a microservice framework written in Rust, similar to SpringBoot in java. <b>spring-rs</b> provides an easily extensible plug-in system for integrating excellent projects in the Rust community, such as axum, sqlx, sea-orm, etc.

Compared with SpringBoot in java, spring-rs has higher performance and lower memory usage, allowing you to completely get rid of the bloated JVM and travel light.

## Example

```rust
use spring::{get, route, auto_config, App};
use spring_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin
};
use spring_web::{
    error::Result, extractor::{Path, Component}, handler::TypeRouter, axum::response::IntoResponse, Router,
    WebConfigurator, WebPlugin,
};
use anyhow::Context;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[get("/")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
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
