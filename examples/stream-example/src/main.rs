use anyhow::Context;
use serde_json::json;
use spring::{auto_config, get, stream_listener, App};
use spring_stream::consumer::{Consumer, Consumers};
use spring_stream::{
    ConsumerMode, FileConsumerOptions, Producer, StreamConfigurator, StreamPlugin,
};
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
    Consumers::new().add_consumer(
        Consumer::mode(ConsumerMode::RealTime)
            .group_id("group_id")
            .file_consumer_options(fill_file_consumer_options)
            .consume(&["stream topic"], listen_topic_do_something),
    )
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

#[stream_listener("topic", "topic2", mode = "RealTime", group_id = "groupId", file_consumer_options = "fill_file_consumer_options")]
async fn listen_topic_do_something() {
    println!("do something");
}

fn fill_file_consumer_options(_opts: &mut FileConsumerOptions) {
    //
}
