[![crates.io](https://img.shields.io/crates/v/spring-sea-orm.svg)](https://crates.io/crates/spring-sea-orm)
[![Documentation](https://docs.rs/spring-sea-orm/badge.svg)](https://docs.rs/spring-sea-orm)

## 依赖

```toml
spring-sea-orm = { version = "0.0.7", features = ["postgres"] }
sea-orm = { version = "1.0" }    # 主要为了适配sea-orm-cli生成的entity代码
```

可以替换`postgres`、`mysql`、`sqlite`feature来选择合适的数据库驱动。

## 配置项

```toml
[sea-orm]
uri = "postgres://root:123456@localhost:5432/pg_db"  # 数据库地址
min_connections = 1                                  # 连接池的最小连接数，默认值为1
max_connections = 10                                 # 连接池的最大连接数，默认值为10
acquire_timeout = 30000                              # 占用连接超时时间，单位毫秒，默认30s
idle_timeout = 600000                                # 连接空闲时间，单位毫秒，默认10min
connect_timeout = 1800000                            # 连接的最大存活时间，单位毫秒，默认30min
enable_logging = true                                # 打印sql日志
```

## 组件

配置完上述配置项后，插件会自动注册一个[`DbConn`](https://docs.rs/spring-sea-orm/latest/spring_sea_orm/type.DbConn.html)连接池对象。该对象是[`sea_orm::DbConn`](https://docs.rs/sea-orm/1.0.0/sea_orm/type.DbConn.html)的别名。

```rust
pub type DbConn = sea_orm::DbConn;
```

## 提取插件注册的Component

`SeaOrmPlugin`插件为我们自动注册了一个连接池组件，我们可以使用`Component`从AppState中提取这个连接池，[`Component`](https://docs.rs/spring-web/latest/spring_web/extractor/struct.Component.html)是一个axum的[extractor](https://docs.rs/axum/latest/axum/extract/index.html)。

```rust
use spring::get;
use spring_sqlx::{sqlx::{self, Row}, ConnectPool};
use spring_web::extractor::Component;
use spring_web::error::Result;
use anyhow::Context;

#[get("/:id")]
async fn get_todo_list(
    Component(db): Component<DbConn>,
    Path(id): Path<i32>
) -> Result<String> {
    let rows = TodoItem::find()
        .filter(todo_item::Column::ListId.eq(id))
        .all(&db)
        .await
        .context("query todo list failed")?;
    Ok(Json(rows))
}
```

完整代码参考[`sea-orm-example`](https://github.com/spring-rs/spring-rs/tree/master/examples/sea-orm-example)