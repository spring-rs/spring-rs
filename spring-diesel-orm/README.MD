[![crates.io](https://img.shields.io/crates/v/spring-diesel-orm.svg)](https://crates.io/crates/spring-diesel-orm)
[![Documentation](https://docs.rs/spring-diesel-orm/badge.svg)](https://docs.rs/spring-diesel-orm)

[Diesel](https://docs.rs/diesel/latest/diesel/) is ORM framework that supports Mysql, Postgres and Sqlite databases using blocking connection. 

[Diesel Async](https://docs.rs/diesel-async/latest/diesel_async/) is asyncrnous wrapper around Diesel.

spring-diesel-orm plugin support the above two implementations.

## Dependencies

```toml
spring-diesel-orm = { version = "<version>", features = ["..."] }
```

*  Following features are available depending on database type, connection pool type and async requirements

    * diesel-async-postgres-deadpool
    * diesel-async-sqlite-deadpool
    * diesel-async-mysql-deadpool
    * diesel-async-postgres-bb8
    * diesel-async-sqlite-bb8
    * diesel-async-mysql-bb8
    * diesel-sync-postgres-r2d2
    * diesel-sync-sqlite-r2d2
    * diesel-sync-mysql-r2d2

## Configuration items

### Configure postgres with async deadpool connection pooling in your app.toml

```toml
[diesel-async]
uri = "postgresql://user:pass@localhost:5432/postgres"
pool_type = "Deadpool"
pool_config = { Deadpool = { max_size = 16,  create_timeout_in_ms=1000, recycle_timeout_in_ms=1000, wait_timeout_in_ms = 1000 } }

```

### Configure postgres with sync r2d2 connection pooling
```
[diesel-sync]
uri = "postgresql://user:pass@localhost:5432/postgres"
pool_type = "r2d2"
pool_config = { R2d2 = { max_size = 16, min_idle= 4,  test_on_check_out = true, max_lifetime_in_ms = 300000, idle_timeout_in_ms=300000, connection_timeout_in_ms=5000} }
```

## Two plugins are supported
    
1. DieselSyncOrmPlugin          - Supports r2d2 synchronous connection 
2. DieselAsyncOrmPlugin         - Supports deadpool and bb8 asynchrnonous connection


use [diesel-sync] in app.toml for DieselSyncOrmPlugin
use [diesel-async] in app.toml for DieselAsyncOrmPlugin

## Components

After configuring the above configuration items, 

#### Extract the Component registered by the plugin

The `DieselASyncOrmPlugin` plugin automatically registers a connection pool component for us. We can use `Component` to extract this connection pool from AppState. [`Component`](https://docs.rs/spring-web/latest/spring_web/extractor/struct.Component.html) is an axum [extractor](https://docs.rs/axum/latest/axum/extract/index.html).

```rust
use spring_web::get;
use spring_web::{
    axum::response::{IntoResponse, Json},
    error::Result,
    extractor::Component,
    WebConfigurator, WebPlugin,
};

use anyhow::Context;

use diesel::prelude::*;
use crate::schema::users;
use diesel_async::{RunQueryDsl};
use spring_diesel_orm::diesel_async::{DieselAsyncOrmPlugin, PgBb8ConnectionPool};
#[get("/users")]
async fn get_users(Component(db): Component<PgBb8ConnectionPool>) -> Result<impl IntoResponse> {
    let mut connection = db.get().await.context("failed to get db connection")?;
    let rows: Vec<User> = users::table
        .filter(users::active.eq(true))
        .limit(10)
        .load(&mut connection)
        .await
        .context("query users failed")?;
    Ok(Json(rows))
}
```

* The following components are registered depending on your configuration in app.toml

    1.   PgDeadPoolConnectionPool
    2.   MysqlDeadPoolConnectionPool
    3.   SqliteDeadPoolConnectionPool
    4.   PgBb8ConnectionPool
    5.   MysqlBb8ConnectionPool  
    6.   SqliteBb8ConnectionPool
    7.   PgR2d2ConnectionPool
    8.   MysqlR2d2ConnectionPool
    9.   SqliteR2d2ConnectionPool

For the complete code, please refer to [`diesel-orm-example`](https://github.com/spring-rs/spring-rs/tree/master/examples/diesel-orm-example)