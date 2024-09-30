<div align="center">
    <img src="docs/static/logo-rust.svg" alt="Logo" width="200"/>
    <h3>spring-rs是Rust编写的应用框架，类似于java生态的SpringBoot</h3>
    <p><a href="./README.md">English</a> ｜ 中文</p>
    <p>
        <a href="https://crates.io/crates/spring">
            <img src="https://img.shields.io/crates/v/spring.svg" alt="crates.io"/>
        </a>
        <a href="https://docs.rs/spring">
            <img src="https://docs.rs/spring/badge.svg" alt="Documentation"/>
        </a>
    </p>
</div>

<b>spring-rs</b>是一个Rust编写的应用框架，强调约定大于服务，类似于java生态的SpringBoot。<b>spring-rs</b>提供了易于扩展的插件系统，用于整合Rust社区的优秀项目，例如axum、sqlx、sea-orm等。

相比于java生态的SpringBoot，spring-rs有更高的性能和更低的内存占用，让你彻底摆脱臃肿的JVM，轻装上阵。

## 简单的例子

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

## 支持的插件

* [x] ![spring-web](https://img.shields.io/crates/v/spring-web.svg)[`spring-web`](./spring-web/README.zh.md)(基于[`axum`](https://github.com/tokio-rs/axum)实现)
* [x] ![spring-sqlx](https://img.shields.io/crates/v/spring-sqlx.svg)[`spring-sqlx`](./spring-sqlx/README.zh.md)(整合了[`sqlx`](https://github.com/launchbadge/sqlx))
* [x] ![spring-postgres](https://img.shields.io/crates/v/spring-postgres.svg)[`spring-postgres`](./spring-postgres/README.zh.md)(整合了[`rust-postgres`](https://github.com/sfackler/rust-postgres))
* [x] ![spring-sea-orm](https://img.shields.io/crates/v/spring-sea-orm.svg)[`spring-sea-orm`](./spring-sea-orm/README.zh.md)(整合了[`sea-orm`](https://www.sea-ql.org/SeaORM/))
* [x] ![spring-redis](https://img.shields.io/crates/v/spring-redis.svg)[`spring-redis`](./spring-redis/README.zh.md)(整合了[`redis`](https://github.com/redis-rs/redis-rs))
* [x] ![spring-mail](https://img.shields.io/crates/v/spring-mail.svg)[`spring-mail`](./spring-mail/README.zh.md)(整合了[`lettre`](https://github.com/lettre/lettre))
* [x] ![spring-job](https://img.shields.io/crates/v/spring-job.svg)[`spring-job`](./spring-job/README.zh.md)(整合了[`tokio-cron-scheduler`](https://github.com/mvniekerk/tokio-cron-scheduler))
* [x] ![spring-stream](https://img.shields.io/crates/v/spring-stream.svg)[`spring-stream`](./spring-stream/README.zh.md)(整合了[`sea-streamer`](https://github.com/SeaQL/sea-streamer)实现redis-stream、kafka等消息处理)
* [ ] `spring-opentelemetry`(整合了[`opentelemetry`](https://github.com/open-telemetry/opentelemetry-rust)实现logging、metrics、tracing全套可观测性)
* [ ] `spring-tarpc`(整合了[`tarpc`](https://github.com/google/tarpc)实现RPC调用)

![star history](https://api.star-history.com/svg?repos=spring-rs/spring-rs&type=Date)

## 请作者喝杯茶

<table>
<tr>
<td><img src="docs/static/sponsor-wechat.jpg" alt="微信" height="400"/></td>
<td><img src="docs/static/sponsor-alipay.jpg" alt="支付宝" height="400"/></td>
</tr>
</table>
