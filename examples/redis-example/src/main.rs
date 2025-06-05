use anyhow::Context;
use spring::{auto_config, App};
use spring_redis::{cache, redis::AsyncCommands, Redis, RedisPlugin};
use spring_web::{
    axum::response::{IntoResponse, Json},
    error::Result,
    extractor::{Component, Path},
    WebConfigurator, WebPlugin,
};
use spring_web::{get, post};

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(RedisPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[get("/")]
async fn list_redis_key(Component(mut redis): Component<Redis>) -> Result<impl IntoResponse> {
    let keys: Vec<String> = redis.keys("*").await.context("redis request failed")?;
    Ok(Json(keys))
}

#[get("/get/{key}")]
async fn get_content(
    Component(mut redis): Component<Redis>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse> {
    let v: String = redis.get(key).await.context("redis request failed")?;
    Ok(v)
}

#[post("/put/{key}")]
async fn set_content(
    Component(mut redis): Component<Redis>,
    Path(key): Path<String>,
    body: String,
) -> Result<impl IntoResponse> {
    let v: String = redis.set(key, body).await.context("redis request failed")?;
    Ok(v)
}

#[get("/cache/{key}")]
async fn test_cache(Path(key): Path<String>) -> Result<impl IntoResponse> {
    Ok(cachable_func(&key).await)
}

#[cache("redis-cache:{key}", expire = 60)]
async fn cachable_func(key: &str) -> String {
    format!("cached value for key: {key}")
}
