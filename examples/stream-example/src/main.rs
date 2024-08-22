use anyhow::Context;
use serde_json::json;
use spring::{auto_config, get, App};
use spring_stream::{Producer, StreamPlugin};
use spring_web::error::Result;
use spring_web::{
    extractor::Component,
    response::{IntoResponse, Json},
    WebConfigurator, WebPlugin,
};
use std::time::SystemTime;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(StreamPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await
}

#[get("/")]
async fn send_msg(Component(producer): Component<Producer>) -> Result<impl IntoResponse> {
    let now = SystemTime::now();
    let resp = producer
        .send_to("topic", format!("This message was sent at {:?}", now))
        .await
        .context("send msg failed")?;

    let seq = resp.sequence();
    Ok(Json(json! {seq}))
}
