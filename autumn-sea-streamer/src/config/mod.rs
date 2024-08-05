pub mod file;
pub mod kafka;
pub mod redis;
pub mod stdio;

use std::time::Duration;

use schemars::JsonSchema;
use sea_streamer::{
    file::{FileConnectOptions, FileStreamer},
    kafka::{KafkaConnectOptions, KafkaStreamer},
    redis::{RedisConnectOptions, RedisStreamer},
    stdio::{StdioConnectOptions, StdioStreamer},
    ConnectOptions, SeaConnectOptions, SeaConsumerOptions, SeaProducerOptions, SeaStreamer,
    Streamer, StreamerUri,
};
use serde::Deserialize;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct StreamConfig {
    /// streamer uri
    /// https://docs.rs/sea-streamer-types/latest/sea_streamer_types/struct.StreamerUri.html
    pub(crate) uri: String,

    #[cfg(feature = "kafka")]
    pub(crate) kafka: Option<kafka::KafkaOptions>,
    #[cfg(feature = "redis")]
    pub(crate) redis: Option<redis::RedisOptions>,
    #[cfg(feature = "stdio")]
    pub(crate) stdio: Option<stdio::StdioOptions>,
    #[cfg(feature = "file")]
    pub(crate) file: Option<file::FileOptions>,
}

impl StreamConfig {
    pub fn connect_options(&self) -> SeaConnectOptions {
        let mut connect_options = SeaConnectOptions::default();

        #[cfg(feature = "kafka")]
        if let Some(kafka) = &self.kafka {
            connect_options.set_kafka_connect_options(|opts| kafka.fill_connect_options(opts));
        }
        #[cfg(feature = "redis")]
        if let Some(redis) = &self.redis {
            connect_options.set_redis_connect_options(|opts| redis.fill_connect_options(opts));
        }
        #[cfg(feature = "stdio")]
        if let Some(stdio) = &self.stdio {
            connect_options.set_stdio_connect_options(|opts| stdio.fill_connect_options(opts));
        }
        #[cfg(feature = "file")]
        if let Some(file) = &self.file {
            connect_options.set_file_connect_options(|opts| file.fill_connect_options(opts));
        }
        connect_options
    }

    pub fn consumer_options(&self) -> SeaConsumerOptions {
        let mut consumer_options = SeaConsumerOptions::default();

        #[cfg(feature = "kafka")]
        if let Some(kafka) = &self.kafka {
            consumer_options.set_kafka_consumer_options(|opts| kafka.fill_consumer_options(opts));
        }
        #[cfg(feature = "redis")]
        if let Some(redis) = &self.redis {
            consumer_options.set_redis_consumer_options(|opts| redis.fill_consumer_options(opts));
        }
        #[cfg(feature = "stdio")]
        if let Some(stdio) = &self.stdio {
            consumer_options.set_stdio_consumer_options(|opts| stdio.fill_consumer_options(opts));
        }
        #[cfg(feature = "file")]
        if let Some(file) = &self.file {
            consumer_options.set_file_consumer_options(|opts| file.fill_consumer_options(opts));
        }
        consumer_options
    }

    pub fn producer_options(&self) -> SeaProducerOptions {
        let mut producer_options = SeaProducerOptions::default();

        #[cfg(feature = "kafka")]
        if let Some(kafka) = &self.kafka {
            producer_options.set_kafka_producer_options(|opts| kafka.fill_producer_options(opts));
        }
        #[cfg(feature = "redis")]
        if let Some(redis) = &self.redis {
            producer_options.set_redis_producer_options(|opts| redis.fill_producer_options(opts));
        }
        #[cfg(feature = "stdio")]
        if let Some(stdio) = &self.stdio {
            producer_options.set_stdio_producer_options(|opts| stdio.fill_producer_options(opts));
        }
        #[cfg(feature = "file")]
        if let Some(file) = &self.file {
            producer_options.set_file_producer_options(|opts| file.fill_producer_options(opts));
        }
        producer_options
    }
}

pub(crate) trait OptionsFiller {
    type ConnectOptsType;
    type ConsumerOptsType;
    type ProducerOptsType;
    fn fill_connect_options(&self, opts: &mut Self::ConnectOptsType);
    fn fill_consumer_options(&self, opts: &mut Self::ConsumerOptsType);
    fn fill_producer_options(&self, opts: &mut Self::ProducerOptsType);
}
