+++
title = "Quick Start"
description = "A page introducing how to quickly get started with spring-rs"
draft = false
weight = 3
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = "On this page, I will introduce how to quickly get started with spring-rs"
toc = true
top = false
+++

## Prepare the environment

* rust ≥ 1.75

## Add dependencies

Add the following dependencies to your `Cargo.toml` file

```toml
[dependencies]
# Spring provides the core plugin system and useful Procedural Macros
spring = "0.0.9"
# If you are going to write a web application, add spring-web
spring-web = "0.0.9"
# If the application needs to interact with the database, add spring-sqlx
spring-sqlx = { version="0.0.9", features = ["mysql"] }
# The spring-rs project uses the tokio asynchronous runtime by default
tokio = "1"
```

## Write code

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

// Main function entry
#[auto_config(WebConfigurator)]   // auto config web router
#[tokio::main]
async fn main() {
    App::new()
    .add_plugin(SqlxPlugin) // Add plug-in
    .add_plugin(WebPlugin)
    .run()
    .await
}

// The get macro specifies the Http Method and request path. 
// spring-rs also provides other standard http method macros such as post, delete, patch, etc.
#[get("/")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
}

// You can also use the route macro to specify the Http Method and request path. 
// Path extracts parameters from the HTTP request path
#[route("/hello/:name", method = "GET", method = "POST")]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("hello {name}")
}
// Component can extract the connection pool registered by the Sqlx plug-in in AppState
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

## Configure the application

Create a `config` directory in the root path of the project, where the `spring-rs` configuration files will be stored.

You can first create an `app.toml` file in this directory with the following content:

```toml
[web]
port = 8000 # Configure the web service port. If not configured, the default port is 8080

[sqlx] # Configure the database connection information of sqlx
uri = "mysql://user:password@127.0.0.1:3306"
```

`spring-rs` supports multiple environment configurations: dev (development), test (testing), and prod (production), corresponding to the three configuration files `app-dev.toml`, `app-dev.toml`, and `app-prod.toml`. The configuration in the environment configuration file will override the configuration items of the `app.toml` main configuration file.

`spring-rs` will activate the configuration file of the corresponding environment according to the `SPRING_ENV` environment variable.

## Run

Coding is complete, please make sure your database can be connected normally, then let's start running.

```sh
cargo run
```