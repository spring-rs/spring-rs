use spring::tracing;
use spring::App;
use spring_stream::consumer::Consumers;
use spring_stream::extractor::{Json, StreamKey};
use spring_stream::handler::TypedConsumer;
use spring_stream::redis::AutoStreamReset;
use spring_stream::stream_listener;
use spring_stream::{redis::RedisConsumerOptions, StreamConfigurator, StreamPlugin};
use stream_redis_example::Payload;

#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(StreamPlugin)
        .add_consumer(consumers())
        .run()
        .await
}

fn consumers() -> Consumers {
    Consumers::new().typed_consumer(listen_topic_do_something)
}

#[stream_listener(
    "topic",
    redis_consumer_options = fill_redis_consumer_options
)]
async fn listen_topic_do_something(topic: StreamKey, Json(payload): Json<Payload>) {
    tracing::info!("received msg from topic#{}: {:#?}", topic, payload);
}

fn fill_redis_consumer_options(opts: &mut RedisConsumerOptions) {
    opts.set_auto_stream_reset(AutoStreamReset::Earliest);
}
