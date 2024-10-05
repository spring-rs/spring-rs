<div align="center">
    <img src="docs/static/logo-rust.svg" alt="Logo" width="200"/>
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

**web**

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

**job**

```rust
use anyhow::Context;
use spring::{auto_config, App};
use spring_job::{cron, fix_delay, fix_rate};
use spring_job::{extractor::Component, JobConfigurator, JobPlugin};
use spring_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin,
};
use std::time::{Duration, SystemTime};

#[auto_config(JobConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(JobPlugin)
        .add_plugin(SqlxPlugin)
        .run()
        .await;

    tokio::time::sleep(Duration::from_secs(100)).await;
}

#[cron("1/10 * * * * *")]
async fn cron_job(Component(db): Component<ConnectPool>) {
    let time: String = sqlx::query("select TO_CHAR(now(),'YYYY-MM-DD HH24:MI:SS') as time")
        .fetch_one(&db)
        .await
        .context("query failed")
        .unwrap()
        .get("time");
    println!("cron scheduled: {:?}", time)
}

#[fix_delay(5)]
async fn fix_delay_job() {
    let now = SystemTime::now();
    let datetime: sqlx::types::chrono::DateTime<sqlx::types::chrono::Local> = now.into();
    let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S");
    println!("fix delay scheduled: {}", formatted_time)
}

#[fix_rate(5)]
async fn fix_rate_job() {
    tokio::time::sleep(Duration::from_secs(10)).await;
    let now = SystemTime::now();
    let datetime: sqlx::types::chrono::DateTime<sqlx::types::chrono::Local> = now.into();
    let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S");
    println!("fix rate scheduled: {}", formatted_time)
}
```

## Supported plugins

* [x] ![spring-web](https://img.shields.io/crates/v/spring-web.svg)[`spring-web`](./spring-web/)(Based on [`axum`](https://github.com/tokio-rs/axum))
* [x] ![spring-sqlx](https://img.shields.io/crates/v/spring-sqlx.svg)[`spring-sqlx`](./spring-sqlx/)(Integrated with [`sqlx`](https://github.com/launchbadge/sqlx))
* [x] ![spring-postgres](https://img.shields.io/crates/v/spring-postgres.svg)[`spring-postgres`](./spring-postgres/)(Integrated with [`rust-postgres`](https://github.com/sfackler/rust-postgres))
* [x] ![spring-sea-orm](https://img.shields.io/crates/v/spring-sea-orm.svg)[`spring-sea-orm`](./spring-sea-orm/)(Integrated with [`sea-orm`](https://www.sea-ql.org/SeaORM/))
* [x] ![spring-redis](https://img.shields.io/crates/v/spring-redis.svg)[`spring-redis`](./spring-redis/)(Integrated with [`redis`](https://github.com/redis-rs/redis-rs))
* [x] ![spring-mail](https://img.shields.io/crates/v/spring-mail.svg)[`spring-mail`](./spring-mail/)(integrated with [`lettre`](https://github.com/lettre/lettre))
* [x] ![spring-job](https://img.shields.io/crates/v/spring-job.svg)[`spring-job`](./spring-job/)(integrated with [`tokio-cron-scheduler`](https://github.com/mvniekerk/tokio-cron-scheduler))
* [x] ![spring-stream](https://img.shields.io/crates/v/spring-stream.svg)[`spring-stream`](./spring-stream/)(Integrate [`sea-streamer`](https://github.com/SeaQL/sea-streamer) to implement message processing such as redis-stream and kafka)
* [ ] `spring-opentelemetry`(integrate with [`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust) to implement full observability of logging, metrics, tracing)
* [ ] `spring-tarpc`(Integrate[`tarpc`](https://github.com/google/tarpc) to implement RPC calls)
