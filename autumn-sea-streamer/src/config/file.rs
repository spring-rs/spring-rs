#![allow(warnings, unused)]

use super::OptionsFiller;
use schemars::JsonSchema;
use sea_streamer::{
    file::{AutoStreamReset, FileConnectOptions, FileConsumerOptions, FileProducerOptions},
    ConsumerGroup, ConsumerMode, ConsumerOptions as ConsumerOptionsTrait,
};
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
        if let Some(connect) = &self.connect {
            // opts.crate_file(connect.create_file);
            opts.set_end_with_eos(connect.end_with_eos);
            opts.set_beacon_interval(connect.beacon_interval);
            opts.set_file_size_limit(connect.file_size_limit);
            opts.set_prefetch_message(connect.prefetch_message);
        }
    }

    fn fill_consumer_options(&self, opts: &mut Self::ConsumerOptsType) {
        if let Some(consumer) = &self.consumer {
            opts.set_auto_stream_reset(consumer.auto_stream_reset);
            opts.set_live_streaming(consumer.live_streaming);
            if let Some(group) = &consumer.group {
                opts.set_consumer_group(ConsumerGroup::new(group));
            }
            //opts.set(consumer.prefetch_message);
        }
    }

    fn fill_producer_options(&self, opts: &mut Self::ProducerOptsType) {
        if let Some(producer) = &self.producer {
            // opts.crate_file(connect.create_file);
            // opts.set_end_with_eos(connect.end_with_eos);
            // opts.set_beacon_interval(connect.beacon_interval);
            // opts.set_file_size_limit(connect.file_size_limit);
            // opts.set_prefetch_message(connect.prefetch_message);
        }
    }
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct ConnectOptions {
    create_file: CreateFileOption,
    end_with_eos: bool,
    beacon_interval: u32,
    file_size_limit: u64,
    prefetch_message: usize,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
enum CreateFileOption {
    /// File must already exists
    Never,
    CreateIfNotExists,
    /// Fail if the file already exists
    Always,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct ConsumerOptions {
    #[serde(with = "ConsumerModeRef")]
    mode: ConsumerMode,
    group: Option<String>,
    #[serde(with = "AutoStreamResetRef")]
    auto_stream_reset: AutoStreamReset,
    live_streaming: bool,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(remote = "ConsumerMode")]
pub enum ConsumerModeRef {
    /// This is the 'vanilla' stream consumer. It does not auto-commit, and thus only consumes messages from now on.
    RealTime,
    /// When the process restarts, it will resume the stream from the previous committed sequence.
    Resumable,
    /// You should assign a consumer group manually. The load-balancing mechanism is implementation-specific.
    LoadBalanced,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(remote = "AutoStreamReset")]
pub enum AutoStreamResetRef {
    Earliest,
    Latest,
}

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct ProducerOptions {}
