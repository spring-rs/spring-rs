use super::OptionsFiller;
use schemars::JsonSchema;
use sea_streamer::stdio::{StdioConnectOptions, StdioConsumerOptions, StdioProducerOptions};
use serde::Deserialize;

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct StdioOptions {
    connect: Option<ConnectOptions>,
    producer: Option<ProducerOptions>,
    consumer: Option<ConsumerOptions>,
}

impl OptionsFiller for StdioOptions {
    type ConnectOptsType = StdioConnectOptions;
    type ConsumerOptsType = StdioConsumerOptions;
    type ProducerOptsType = StdioProducerOptions;

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
