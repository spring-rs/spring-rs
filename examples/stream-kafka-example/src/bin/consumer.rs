use spring::tracing;
use spring::{stream_listener, App};
use spring_stream::consumer::Consumers;
use spring_stream::extractor::{Json, StreamKey};
use spring_stream::handler::TypedConsumer;
use spring_stream::{kafka::KafkaConsumerOptions, StreamConfigurator, StreamPlugin};
use stream_kafka_example::Payload;

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
    kafka_consumer_options = fill_kafka_consumer_options
)]
async fn listen_topic_do_something(topic: StreamKey, Json(payload): Json<Payload>) {
    tracing::info!("received msg from topic#{}: {:#?}", topic, payload);
}

fn fill_kafka_consumer_options(opts: &mut KafkaConsumerOptions) {
    opts.set_enable_auto_offset_store(true);
}
