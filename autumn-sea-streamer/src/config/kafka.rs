use super::OptionsFiller;
use schemars::JsonSchema;
use sea_streamer::kafka::{KafkaConnectOptions, KafkaConsumerOptions, KafkaProducerOptions};
use serde::Deserialize;
use std::time::Duration;

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct KafkaOptions {
    connect: Option<ConnectOptions>,
    producer: Option<ProducerOptions>,
    consumer: Option<ConsumerOptions>,
}

impl OptionsFiller for KafkaOptions {
    type ConnectOptsType = KafkaConnectOptions;
    type ConsumerOptsType = KafkaConsumerOptions;
    type ProducerOptsType = KafkaProducerOptions;

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

#[derive(Debug, Clone, JsonSchema, Deserialize)]
struct ConnectOptions {
    timeout: Option<Duration>,
    security_protocol: Option<SecurityProtocol>,
    sasl_options: Option<SaslOptions>,
    custom_options: Vec<(String, String)>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
struct SaslOptions {
    mechanism: SaslMechanism,
    username: Option<String>,
    password: Option<String>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
enum SaslMechanism {
    Plain,
    Gssapi,
    ScramSha256,
    ScramSha512,
    Oauthbearer,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
enum SecurityProtocol {
    Plaintext,
    Ssl,
    SaslPlaintext,
    SaslSsl,
}

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
struct ProducerOptions {}

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
struct ConsumerOptions {}
