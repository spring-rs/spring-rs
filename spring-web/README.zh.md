[![crates.io](https://img.shields.io/crates/v/spring-web.svg)](https://crates.io/crates/spring-web)
[![Documentation](https://docs.rs/spring-web/badge.svg)](https://docs.rs/spring-web)

## 依赖

```toml
spring-web = { version = "0.0.7" }
```

## 配置项

```toml
[web]
binding = "172.20.10.4"  # 要绑定的网卡IP地址，默认127.0.0.1
port = 8000              # 要绑定的端口号，默认8080

# web中间件配置
[web.middlewares]
compression = { enable = true }  # 开启压缩中间件
logger = { enable = true }       # 开启日志中间件
catch_panic = { enable = true }  # 捕获handler产生的panic
limit_payload = { enable = true, body_limit = "5MB" } # 限制请求体大小
timeout_request = { enable = true, timeout = 60000 }  # 请求超时时间60s

# 跨域配置
cors = { enable = true, allow_origins = [
    "*.github.io",
], allow_headers = [
    "Authentication",
], allow_methods = [
    "GET",
    "POST",
], max_age = 60 }

# 静态资源配置
static = { enable = true, uri = "/static", path = "static", precompressed = true, fallback = "index.html" }
```

## API接口

App实现了[WebConfigurator](https://docs.rs/spring-web/latest/spring_web/trait.WebConfigurator.html)特征，可以通过该特征指定路由配置：

```diff
 #[tokio::main]
 async fn main() {
     App::new()
         .add_plugin(SqlxPlugin)
         .add_plugin(WebPlugin)
+        .add_router(router())
         .run()
         .await
 }

+fn router() -> Router {
+    Router::new()
+        .typed_route(hello_word)
+}

+#[get("/")]
+async fn hello_word() -> impl IntoResponse {
+    "hello word"
+}
```

你也可以使用`auto_config`宏来实现自动配置，这个过程宏会自动将被过程宏标记的路由注册进app中：

```diff
+#[auto_config(WebConfigurator)]
 #[tokio::main]
 async fn main() {
     App::new()
         .add_plugin(SqlxPlugin)
         .add_plugin(WebPlugin)
-        .add_router(router())
         .run()
         .await
}
```

## 属性宏

上面例子中的[`get`](https://docs.rs/spring-macros/latest/spring_macros/attr.get.html)是一个属性宏，spring提供了八个标准HTTP METHOD的过程宏：`get`、`post`、`patch`、`put`、`delete`、`head`、`trace`、`options`。

也可以使用[`route`](https://docs.rs/spring-macros/latest/spring_macros/attr.route.html)宏同时绑定多个method：

```rust
#[route("/test", method = "GET", method = "HEAD")]
async fn example() -> impl IntoResponse {
    "hello world"
}
```

除此之外，spring还支持一个handler绑定多个路由，这需要用到[`routes`](https://docs.rs/spring-macros/latest/spring_macros/attr.routes.html)属性宏：

```rust
#[routes]
#[get("/test")]
#[get("/test2")]
#[delete("/test")]
async fn example() -> impl IntoResponse {
    "hello world"
}
```

## 提取插件注册的Component

上面的例子中`SqlxPlugin`插件为我们自动注册了一个Sqlx连接池组件，我们可以使用`Component`从State中提取这个连接池，[`Component`](https://docs.rs/spring-web/latest/spring_web/extractor/struct.Component.html)是一个axum的[extractor](https://docs.rs/axum/latest/axum/extract/index.html)。

```rust
use spring::get;
use spring_sqlx::{sqlx::{self, Row}, ConnectPool};
use spring_web::extractor::Component;
use spring_web::error::Result;
use anyhow::Context;

#[get("/version")]
async fn mysql_version(Component(pool): Component<ConnectPool>) -> Result<String> {
    let version = sqlx::query("select version() as version")
        .fetch_one(&pool)
        .await
        .context("sqlx query failed")?
        .get("version");
    Ok(version)
}
```

axum也提供了其他的[extractor](https://docs.rs/axum/latest/axum/extract/index.html)，这些都被reexport到了[`spring_web::extractor`](https://docs.rs/spring-web/latest/spring_web/extractor/index.html)下。

完整代码参考[`web-example`](https://github.com/spring-rs/spring-rs/tree/master/examples/web-example)