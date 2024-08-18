<div align="center">
    <img src="docs/static/logo.svg" alt="Logo"/>
    <h3>spring-rs是Rust编写的微服务框架</h3>
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

<b>spring-rs</b>是一个Rust编写的微服务框架，类似于java生态的SpringBoot。<b>spring-rs</b>提供了易于扩展的插件系统，用于整合Rust社区的优秀项目，例如axum、sqlx、sea-orm等。

相比于java生态的SpringBoot，spring-rs有更高的性能和更低的内存占用，让你彻底摆脱臃肿的JVM，轻装上阵。

## 简单的例子

```rust
use spring::{nest, route, routes, auto_config, App};
use spring_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin
};
use spring_web::{
    error::Result, extractor::Path, handler::TypeRouter, response::IntoResponse, Router, 
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
