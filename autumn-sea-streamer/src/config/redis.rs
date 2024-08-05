use super::OptionsFiller;
use schemars::JsonSchema;
use sea_streamer::redis::{RedisConnectOptions, RedisConsumerOptions, RedisProducerOptions};
use serde::Deserialize;

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct RedisOptions {
    connect: Option<ConnectOptions>,
    producer: Option<ProducerOptions>,
    consumer: Option<ConsumerOptions>,
}

impl OptionsFiller for RedisOptions {
    type ConnectOptsType = RedisConnectOptions;
    type ConsumerOptsType = RedisConsumerOptions;
    type ProducerOptsType = RedisProducerOptions;

    fn fill_connect_options(&self, opts: &mut Self::ConnectOptsType) {
        todo!()
    }

    fn fill_consumer_options(&self, opts: &mut Self::ConsumerOptsType) {
        todo!()
    }

    fn fill_producer_options(&self, opts: &mut Self::ProducerOptsType) {
        todo!()
    }
}

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct ConnectOptions {}

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct ProducerOptions {}

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct ConsumerOptions {}
