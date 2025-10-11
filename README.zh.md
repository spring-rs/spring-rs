<div align="center">
    <img src="https://raw.githubusercontent.com/spring-rs/spring-rs/refs/heads/master/docs/static/logo-rust.svg" alt="Logo" width="200"/>
    <h3>spring-rs是Rust编写的应用框架，类似于java生态的SpringBoot</h3>
    <p><a href="https://spring-rs.github.io/docs/getting-started/introduction/">English</a> ｜ 中文</p>
    <p>
        <a href="https://crates.io/crates/spring"><img src="https://img.shields.io/crates/v/spring.svg" alt="crates.io"/></a> <a href="https://docs.rs/spring"><img src="https://docs.rs/spring/badge.svg" alt="Documentation"/></a> <img src="https://img.shields.io/crates/l/spring" alt="Documentation"/>
    </p>
</div>

<b>spring-rs</b>是一个Rust编写的应用框架，强调约定大于配置，类似于java生态的SpringBoot。<b>spring-rs</b>提供了易于扩展的插件系统，用于整合Rust社区的优秀项目，例如axum、sqlx、sea-orm等。

相比于java生态的SpringBoot，spring-rs有更高的性能和更低的内存占用，让你彻底摆脱臃肿的JVM，轻装上阵。

## 特点

* ⚡️ 高性能: 得益于出色的Rust语言，<b>spring-rs</b>拥有与c/c++媲美的极致性能
* 🛡️ 高安全性: 相比C/C++，<b>spring-rs</b>使用的Rust语言提供了内存安全和线程安全的能力
* 🔨 轻量级: <b>spring-rs</b>的核心代码不超过5000行，打包的release版二进制文件也非常小巧
* 🔧 容易使用: <b>spring-rs</b>提供了清晰明了的API和可选的过程宏来简化开发
* 🔌 高可扩展性: <b>spring-rs</b>采用高扩展性的插件模式，用户可以自定义插件扩展程序功能
* ⚙️ 高可配置性: <b>spring-rs</b>用toml配置应用和插件，提升应用灵活性

## 简单的例子

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

**任务调度**

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

## 支持的插件

* [x] ![spring-web](https://img.shields.io/crates/v/spring-web.svg)[`spring-web`](./spring-web/README.zh.md)(基于[`axum`](https://github.com/tokio-rs/axum)实现)
* [x] ![spring-sqlx](https://img.shields.io/crates/v/spring-sqlx.svg)[`spring-sqlx`](./spring-sqlx/README.zh.md)(整合了[`sqlx`](https://github.com/launchbadge/sqlx))
* [x] ![spring-postgres](https://img.shields.io/crates/v/spring-postgres.svg)[`spring-postgres`](./spring-postgres/README.zh.md)(整合了[`rust-postgres`](https://github.com/sfackler/rust-postgres))
* [x] ![spring-sea-orm](https://img.shields.io/crates/v/spring-sea-orm.svg)[`spring-sea-orm`](./spring-sea-orm/README.zh.md)(整合了[`sea-orm`](https://www.sea-ql.org/SeaORM/))
* [x] ![spring-redis](https://img.shields.io/crates/v/spring-redis.svg)[`spring-redis`](./spring-redis/README.zh.md)(整合了[`redis`](https://github.com/redis-rs/redis-rs))
* [x] ![spring-mail](https://img.shields.io/crates/v/spring-mail.svg)[`spring-mail`](./spring-mail/README.zh.md)(整合了[`lettre`](https://github.com/lettre/lettre))
* [x] ![spring-job](https://img.shields.io/crates/v/spring-job.svg)[`spring-job`](./spring-job/README.zh.md)(整合了[`tokio-cron-scheduler`](https://github.com/mvniekerk/tokio-cron-scheduler))
* [x] ![spring-stream](https://img.shields.io/crates/v/spring-stream.svg)[`spring-stream`](./spring-stream/README.zh.md)(整合了[`sea-streamer`](https://github.com/SeaQL/sea-streamer)实现redis-stream、kafka等消息处理)
* [x] ![spring-opentelemetry](https://img.shields.io/crates/v/spring-opentelemetry.svg)[`spring-opentelemetry`](./spring-opentelemetry/README.zh.md)(整合了[`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust)实现logging、metrics、tracing全套可观测性)
* [x] ![spring-grpc](https://img.shields.io/crates/v/spring-grpc.svg)[`spring-grpc`](./spring-grpc/README.zh.md)(整合了[`tonic`](https://github.com/hyperium/tonic)实现gRPC调用)
* [x] ![spring-opendal](https://img.shields.io/crates/v/spring-opendal.svg)[`spring-opendal`](https://spring-rs.github.io/docs/plugins/spring-opendal/): 整合[`opendal`](https://github.com/apache/opendal)实现统一的数据访问层

## 生态

* ![spring-sqlx-migration-plugin](https://img.shields.io/crates/v/spring-sqlx-migration-plugin.svg) [`spring-sqlx-migration-plugin`](https://github.com/Phosphorus-M/spring-sqlx-migration-plugin)

[更多>>](https://crates.io/crates/spring/reverse_dependencies)

<img alt="star history" src="https://api.star-history.com/svg?repos=spring-rs/spring-rs&type=Timeline" style="width: 100%"/>

## 项目示例

* [Raline](https://github.com/ralinejs/raline)
* [AutoWDS](https://github.com/AutoWDS/autowds-backend)

## 请作者喝杯茶

<table>
<tr>
<td><img src="https://github.com/user-attachments/assets/fe69c992-2da3-409e-9f61-507be436baeb" alt="微信" height="400"/></td>
<td><img src="https://github.com/user-attachments/assets/25668103-f41e-482f-925f-0007c40a917d" alt="支付宝" height="400"/></td>
</tr>
</table>

## 交流群


<table>
<tr>
<td><img src="https://github.com/user-attachments/assets/f9f2abcb-8d91-4aa1-a8f6-93e789339e45" alt="QQ交流群" height="400"/></td>
<td><img src="https://github.com/user-attachments/assets/d52abd39-be68-468f-8153-93c3e54ff72f" alt="微信交流群" height="400"/></td>
</tr>
</table>

## 贡献

也欢迎社区的大牛贡献自己的插件。 [Contributing →](https://github.com/spring-rs/spring-rs)

## 帮助

点击这里可以查看`spring-rs`使用过程中遇到的常见问题 [Help →](https://spring-rs.github.io/zh/docs/help/faq/)
