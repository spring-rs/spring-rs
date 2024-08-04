use anyhow::Context;
use autumn::App;
use autumn_redis::{redis::AsyncCommands, Redis, RedisPlugin};
use autumn_web::{
    error::Result,
    extractor::{Component, Path},
    get,
    response::{IntoResponse, Json},
    Router, WebConfigurator, WebPlugin,
};

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(RedisPlugin)
        .add_plugin(WebPlugin)
        .add_router(router())
        .run()
        .await
}

fn router() -> Router {
    Router::new()
        .route("/", get(list_redis_key))
        .route("/:key", get(get_content).post(set_content))
}

async fn list_redis_key(Component(mut redis): Component<Redis>) -> Result<impl IntoResponse> {
    let keys: Vec<String> = redis.keys("*").await.context("redis request failed")?;
    Ok(Json(keys))
}

async fn get_content(
    Component(mut redis): Component<Redis>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse> {
    let v: String = redis.get(key).await.context("redis request failed")?;
    Ok(v)
}

async fn set_content(
    Component(mut redis): Component<Redis>,
    Path(key): Path<String>,
    body: String,
) -> Result<impl IntoResponse> {
    let v: String = redis.set(key, body).await.context("redis request failed")?;
    Ok(v)
}
