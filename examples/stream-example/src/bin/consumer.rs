use spring::tracing;
use spring::{stream_listener, App};
use spring_stream::consumer::Consumers;
use spring_stream::extractor::Json;
use spring_stream::file::AutoStreamReset;
use spring_stream::handler::TypedConsumer;
use spring_stream::{file::FileConsumerOptions, StreamConfigurator, StreamPlugin};
use stream_example::Payload;

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
    "topic2",
    file_consumer_options = fill_file_consumer_options
)]
async fn listen_topic_do_something(Json(payload): Json<Payload>) {
    tracing::info!("{:#?}", payload);
}

fn fill_file_consumer_options(opts: &mut FileConsumerOptions) {
    opts.set_auto_stream_reset(AutoStreamReset::Earliest);
}
