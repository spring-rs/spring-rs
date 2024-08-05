use super::OptionsFiller;
use schemars::JsonSchema;
use sea_streamer::file::{FileConnectOptions, FileConsumerOptions, FileProducerOptions};
use serde::Deserialize;

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct FileOptions {
    connect: Option<ConnectOptions>,
    producer: Option<ProducerOptions>,
    consumer: Option<ConsumerOptions>,
}

impl OptionsFiller for FileOptions {
    type ConnectOptsType = FileConnectOptions;
    type ConsumerOptsType = FileConsumerOptions;
    type ProducerOptsType = FileProducerOptions;

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
