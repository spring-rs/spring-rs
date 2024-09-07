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
{{ include_code(path="../../examples/hello-world-example/src/main.rs") }}
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