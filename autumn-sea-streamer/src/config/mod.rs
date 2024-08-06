pub mod file;
pub mod kafka;
pub mod redis;
pub mod stdio;

use schemars::JsonSchema;
use sea_streamer::{
    ConsumerMode, ConsumerOptions, SeaConnectOptions, SeaConsumerOptions, SeaProducerOptions,
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

    pub fn consumer_options(&self, mut consumer_options: SeaConsumerOptions) -> SeaConsumerOptions {
        #[cfg(feature = "kafka")]
        if let Some(kafka) = &self.kafka {
            consumer_options.set_kafka_consumer_options(|opts| kafka.fill_consumer_options(opts))
        }
        #[cfg(feature = "redis")]
        if let Some(redis) = &self.redis {
            consumer_options.set_redis_consumer_options(|opts| redis.fill_consumer_options(opts))
        }
        #[cfg(feature = "stdio")]
        if let Some(stdio) = &self.stdio {
            consumer_options.set_stdio_consumer_options(|opts| stdio.fill_consumer_options(opts))
        }
        #[cfg(feature = "file")]
        if let Some(file) = &self.file {
            consumer_options.set_file_consumer_options(|opts| file.fill_consumer_options(opts))
        }
        consumer_options
    }

    pub fn fill_producer_options(
        &self,
        mut producer_options: SeaProducerOptions,
    ) -> SeaProducerOptions {
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
    fn default_consumer_options(&self) -> SeaConsumerOptions;
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(remote = "ConsumerMode")]
pub(crate) enum ConsumerModeRef {
    /// This is the 'vanilla' stream consumer. It does not auto-commit, and thus only consumes messages from now on.
    RealTime,
    /// When the process restarts, it will resume the stream from the previous committed sequence.
    Resumable,
    /// You should assign a consumer group manually. The load-balancing mechanism is implementation-specific.
    LoadBalanced,
}
