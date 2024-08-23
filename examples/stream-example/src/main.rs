use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;
use spring::{auto_config, get, stream_listener, App};
use spring_stream::consumer::Consumers;
use spring_stream::extractor::Json as JsonExtract;
use spring_stream::handler::TypedConsumer;
use spring_stream::{FileConsumerOptions, Producer, StreamConfigurator, StreamPlugin};
use spring_web::error::Result;
use spring_web::{
    axum::response::{IntoResponse, Json},
    extractor::Component,
    WebConfigurator, WebPlugin,
};
use std::time::SystemTime;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(StreamPlugin)
        .add_plugin(WebPlugin)
        .add_consumer(consumers())
        .run()
        .await
}

fn consumers() -> Consumers {
    Consumers::new().typed_consumer(listen_topic_do_something)
}

#[get("/")]
async fn send_msg(Component(producer): Component<Producer>) -> Result<impl IntoResponse> {
    let now = SystemTime::now();
    let json = json!({
        "success": true,
        "msg": format!("This message was sent at {:?}", now),
    });
    let resp = producer
        .send_json("topic", json)
        .await
        .context("send msg failed")?;

    let seq = resp.sequence();
    Ok(Json(json! {seq}))
}

#[derive(Debug, Serialize, Deserialize)]
struct Payload {
    success: bool,
    msg: String,
}

#[stream_listener(
    "topic",
    "topic2",
    mode = "RealTime",
    group_id = "groupId",
    file_consumer_options = fill_file_consumer_options
)]
async fn listen_topic_do_something(JsonExtract(payload): JsonExtract<Payload>) {
    println!("{:#?}", payload);
}

fn fill_file_consumer_options(_opts: &mut FileConsumerOptions) {
    //
}
