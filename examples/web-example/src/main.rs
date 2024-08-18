use spring::{auto_config, nest, route, routes, App};
use spring_sqlx::SqlxPlugin;
use spring_web::{extractor::Path, response::IntoResponse, WebConfigurator, WebPlugin};

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[routes]
#[get("/")]
#[get("/hello_world")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
}

#[route("/hello/:name", method = "GET", method = "POST")]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("hello {name}")
}

#[nest("/sql")]
mod sql {
    use std::ops::Deref;

    use anyhow::Context;
    use spring::get;
    use spring_sqlx::{
        sqlx::{self, Row},
        ConnectPool,
    };
    use spring_web::error::Result;
    use spring_web::extractor::Component;

    #[get("/version")]
    pub async fn sqlx_request_handler(Component(pool): Component<ConnectPool>) -> Result<String> {
        let version = sqlx::query("select version() as version")
            .fetch_one(&pool)
            .await
            .context("sqlx query failed")?
            .get("version");
        Ok(version)
    }

    #[get("/now")]
    pub async fn sqlx_time_handler(pool: Component<ConnectPool>) -> Result<String> {
        let time = sqlx::query("select DATE_FORMAT(now(),'%Y-%m-%d %H:%i:%s') as time")
            .fetch_one(pool.deref())
            .await
            .context("sqlx query failed")?
            .get("time");
        Ok(time)
    }
}
