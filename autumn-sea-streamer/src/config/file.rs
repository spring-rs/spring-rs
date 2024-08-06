#![allow(warnings, unused)]

use super::OptionsFiller;
use crate::config::ConsumerModeRef;
use schemars::JsonSchema;
use sea_streamer::{
    file::{AutoStreamReset, FileConnectOptions, FileConsumerOptions, FileProducerOptions},
    ConsumerGroup, ConsumerMode, ConsumerOptions as ConsumerOptionsTrait, SeaConsumerOptions,
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
            match connect.create_file {
                CreateFileOption::Always => opts.set_create_only(true),
                CreateFileOption::Never => opts.set_create_if_not_exists(false),
                CreateFileOption::CreateIfNotExists => opts.set_create_if_not_exists(true),
            };
            opts.set_end_with_eos(connect.end_with_eos);
            let _ = opts.set_beacon_interval(connect.beacon_interval * 1024);
            opts.set_file_size_limit(connect.file_size_limit);
            opts.set_prefetch_message(connect.prefetch_message);
        }
    }

    fn default_consumer_options(&self) -> SeaConsumerOptions {
        match &self.consumer {
            Some(consumer) => consumer.mode,
            None => ConsumerMode::default(),
        };

        todo!()
    }

    fn fill_consumer_options(&self, opts: &mut Self::ConsumerOptsType) {
        if let Some(consumer) = &self.consumer {
            opts.set_auto_stream_reset(consumer.auto_stream_reset);
            opts.set_live_streaming(consumer.live_streaming);
            if let Some(group_id) = &consumer.group_id {
                let _ = opts.set_consumer_group(ConsumerGroup::new(group_id));
            }
        }
    }

    fn fill_producer_options(&self, _opts: &mut Self::ProducerOptsType) {
        // no ops
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
    group_id: Option<String>,
    #[serde(with = "AutoStreamResetRef")]
    auto_stream_reset: AutoStreamReset,
    live_streaming: bool,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(remote = "AutoStreamReset")]
pub enum AutoStreamResetRef {
    Earliest,
    Latest,
}

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct ProducerOptions {}
