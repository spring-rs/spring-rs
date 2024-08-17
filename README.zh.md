<div style="text-align:center">
![Logo](docs/static/logo.svg)
<h3>spring-rs是rust编写的微服务框架</h3>
</div>

<b>spring-rs</b>是一个rust编写的微服务框架，类似于java生态的springboot。<b>spring-rs</b>提供了易于扩展的插件系统，用于整合rust社区的优秀项目，例如axum、sqlx、sea-orm等。

相比于java生态的springboot，spring-rs有更高的性能和更低的内存占用，让你彻底摆脱臃肿的JVM，轻装上阵。

## 简单的例子

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
