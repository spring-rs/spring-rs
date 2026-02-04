<div align="center">
    <img src="https://raw.githubusercontent.com/spring-rs/spring-rs/refs/heads/master/docs/static/logo-rust.svg" alt="Logo" width="200"/>
    <h3>spring-rs is application framework written in Rust, inspired by Java's SpringBoot</h3>
    <p>English ｜ <a href="https://spring-rs.github.io/zh/docs/getting-started/introduction/">中文</a></p>
    <p>
        <a href="https://crates.io/crates/spring"><img src="https://img.shields.io/crates/v/spring.svg" alt="crates.io"/></a> <a href="https://docs.rs/spring"><img src="https://docs.rs/spring/badge.svg" alt="Documentation"/></a> <img src="https://img.shields.io/crates/l/spring" alt="Documentation"/>
    </p>
</div>

<b>spring-rs</b> is an application framework that emphasizes convention over configuration, inspired by Java's SpringBoot. <b>spring-rs</b> provides an easily extensible plug-in system for integrating excellent projects in the Rust community, such as axum, sqlx, sea-orm, etc.

Compared with SpringBoot in java, spring-rs has higher performance and lower memory usage, allowing you to completely get rid of the bloated JVM and travel light.

## Features

* ⚡️ High performance: Benefiting from the awesome rust language, <b>spring-rs</b> has the ultimate performance comparable to C/C++
* 🛡️ High security: Compared to C/C++, the Rust language used by <b>spring-rs</b> provides memory safety and thread safety.
* 🔨 Lightweight: The core code of spring-rs does not exceed 5,000 lines, and the binary size of the release version packaged in rust is also small.
* 🔧 Easy to use: <b>spring-rs</b> provides a clear and concise API and optional Procedural Macros to simplify development.
* 🔌 Highly extensible: <b>spring-rs</b> uses a highly extensible plug-in model, and users can customize plug-ins to extend program capabilities.
* ⚙️ Highly configurable: <b>spring-rs</b> uses toml to configure applications and plug-ins to improve application flexibility.

## Example

**web**

```rust,no_run
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

#[route("/hello/{name}", method = "GET", method = "POST")]
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

```rust,no_run
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

| Plugin                | Crate                                                                                                                                                                      | Integrated With                                                               | Description                                      |
| --------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------- | ------------------------------------------------ |
| `spring-web`            | [![spring-web](https://img.shields.io/crates/v/spring-web.svg)](https://spring-rs.github.io/docs/plugins/spring-web/)                                         | [`axum`](https://github.com/tokio-rs/axum)                                  | Web framework based on Axum                      |
| `spring-sqlx`           | [![spring-sqlx](https://img.shields.io/crates/v/spring-sqlx.svg)](https://spring-rs.github.io/docs/plugins/spring-sqlx/)                                     | [`sqlx`](https://github.com/launchbadge/sqlx)                               | Async SQL access                                 |
| `spring-postgres`       | [![spring-postgres](https://img.shields.io/crates/v/spring-postgres.svg)](https://spring-rs.github.io/docs/plugins/spring-postgres/)                     | [`rust-postgres`](https://github.com/sfackler/rust-postgres)                | PostgreSQL client integration                   |
| `spring-sea-orm`        | [![spring-sea-orm](https://img.shields.io/crates/v/spring-sea-orm.svg)](https://spring-rs.github.io/docs/plugins/spring-sea-orm/)                         | [`sea-orm`](https://www.sea-ql.org/SeaORM/)                                 | ORM support                                      |
| `spring-redis`          | [![spring-redis](https://img.shields.io/crates/v/spring-redis.svg)](https://spring-rs.github.io/docs/plugins/spring-redis/)                                 | [`redis`](https://github.com/redis-rs/redis-rs)                             | Redis integration                                |
| `spring-mail`           | [![spring-mail](https://img.shields.io/crates/v/spring-mail.svg)](https://spring-rs.github.io/docs/plugins/spring-mail/)                                     | [`lettre`](https://github.com/lettre/lettre)                                | Email sending                                    |
| `spring-job`            | [![spring-job](https://img.shields.io/crates/v/spring-job.svg)](https://spring-rs.github.io/docs/plugins/spring-job/)                                         | [`tokio-cron-scheduler`](https://github.com/mvniekerk/tokio-cron-scheduler) | Scheduled jobs / Cron                            |
| `spring-stream`         | [![spring-stream](https://img.shields.io/crates/v/spring-stream.svg)](https://spring-rs.github.io/docs/plugins/spring-stream/)                             | [`sea-streamer`](https://github.com/SeaQL/sea-streamer)                     | Stream processing (Redis Streams / Kafka)       |
| `spring-opentelemetry`  | [![spring-opentelemetry](https://img.shields.io/crates/v/spring-opentelemetry.svg)](https://spring-rs.github.io/docs/plugins/spring-opentelemetry/) | [`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust)     | Logging, metrics, and distributed tracing        |
| `spring-grpc`           | [![spring-grpc](https://img.shields.io/crates/v/spring-grpc.svg)](https://spring-rs.github.io/docs/plugins/spring-grpc/)                                     | [`tonic`](https://github.com/hyperium/tonic)                                | gRPC services and clients                        |
| `spring-opendal`        | [![spring-opendal](https://img.shields.io/crates/v/spring-opendal.svg)](https://spring-rs.github.io/docs/plugins/spring-opendal/)                         | [`opendal`](https://github.com/apache/opendal)                              | Unified object storage and data access           |
| `spring-apalis`        | [![spring-apalis](https://img.shields.io/crates/v/spring-apalis.svg)](https://spring-rs.github.io/docs/plugins/spring-apalis/)                         | [`apalis`](https://github.com/apalis-dev/apalis)                              | High-performance background processing library |
| `spring-sa-token`      | [![spring-sa-token](https://img.shields.io/crates/v/spring-sa-token.svg)](https://spring-rs.github.io/docs/plugins/spring-sa-token/)               | [`sa-token-rust`](https://github.com/click33/sa-token-rust)                   | Sa-Token authentication and authorization      |

## Ecosystem

* ![spring-sqlx-migration-plugin](https://img.shields.io/crates/v/spring-sqlx-migration-plugin.svg) [`spring-sqlx-migration-plugin`](https://github.com/Phosphorus-M/spring-sqlx-migration-plugin)
* [![JetBrains Plugin](https://img.shields.io/badge/JetBrains-Plugin-orange)](https://plugins.jetbrains.com/plugin/30040-spring-rs) [`intellij-spring-rs`](https://github.com/ouywm/intellij-spring-rs) - IDE support for RustRover / IntelliJ IDEA

[more>>](https://crates.io/crates/spring/reverse_dependencies)

## Project showcase

* [Raline](https://github.com/ralinejs/raline)
* [AutoWDS](https://github.com/AutoWDS/autowds-backend)

## Contribution

We also welcome community experts to contribute their own plugins. [Contributing →](https://github.com/spring-rs/spring-rs)

## Help

Click here to view common problems encountered when using `spring-rs` [Help →](https://spring-rs.github.io/docs/help/faq/)
