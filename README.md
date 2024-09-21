<div align="center">
    <img src="docs/static/logo.svg" alt="Logo"/>
    <h3>spring-rs is application framework written in Rust, inspired by Java's SpringBoot</h3>
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

<b>spring-rs</b> is an application framework that emphasizes convention over configuration, inspired by Java's SpringBoot. <b>spring-rs</b> provides an easily extensible plug-in system for integrating excellent projects in the Rust community, such as axum, sqlx, sea-orm, etc.

Compared with SpringBoot in java, spring-rs has higher performance and lower memory usage, allowing you to completely get rid of the bloated JVM and travel light.

## Example

```rust
use spring::{auto_config, App};
use spring_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin
};
use spring_web::{get, route};
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

## Supported plugins

* [x] [`spring-web`](/docs/plugins/spring-web/)(Based on [`axum`](https://github.com/tokio-rs/axum))
* [x] [`spring-sqlx`](/docs/plugins/spring-sqlx/)(Integrated with [`sqlx`](https://github.com/launchbadge/sqlx))
* [x] [`spring-postgres`](/docs/plugins/spring-postgres/)(Integrated with [`rust-postgres`](https://github.com/sfackler/rust-postgres))
* [x] [`spring-sea-orm`](/docs/plugins/spring-sea-orm/)(Integrated with [`sea-orm`](https://www.sea-ql.org/SeaORM/))
* [x] [`spring-redis`](/docs/plugins/spring-redis/)(Integrated with [`redis`](https://github.com/redis-rs/redis-rs))
* [x] [`spring-mail`](/docs/plugins/spring-mail/)(integrated with [`lettre`](https://github.com/lettre/lettre))
* [x] [`spring-job`](/docs/plugins/spring-job/)(integrated with [`tokio-cron-scheduler`](https://github.com/mvniekerk/tokio-cron-scheduler))
* [x] [`spring-stream`](/docs/plugins/spring-stream/)(Integrate [`sea-streamer`](https://github.com/SeaQL/sea-streamer) to implement message processing such as redis-stream and kafka)
* [ ] `spring-opentelemetry`(integrate with [`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust) to implement full observability of logging, metrics, tracing)
* [ ] `spring-tarpc`(Integrate[`tarpc`](https://github.com/google/tarpc) to implement RPC calls)
