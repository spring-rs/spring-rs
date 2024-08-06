use std::time::Duration;

use crate::config::ConsumerModeRef;

use super::OptionsFiller;
use schemars::JsonSchema;
use sea_streamer::redis::{
    AutoCommit, AutoStreamReset, RedisConnectOptions, RedisConsumerOptions, RedisProducerOptions,
    ShardOwnership,
};
use sea_streamer::{
    ConnectOptions as ConnectOptionsTrait, ConsumerMode, ConsumerOptions as ConsumerOptionsTrait,
};
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
        if let Some(connect) = &self.connect {
            opts.set_db(connect.db);
            opts.set_username(connect.username.clone());
            opts.set_password(connect.password.clone());
            opts.set_enable_cluster(connect.enable_cluster);
            opts.set_disable_hostname_verification(connect.disable_hostname_verification);
            if let Some(timeout) = connect.timeout {
                let _ = opts.set_timeout(timeout);
            }
        }
    }

    fn default_consumer_options(&self) -> sea_streamer::SeaConsumerOptions {
        match &self.consumer {
            Some(consumer) => consumer.mode,
            None => ConsumerMode::default(),
        };
        todo!()
    }

    fn fill_consumer_options(&self, opts: &mut Self::ConsumerOptsType) {
        if let Some(consumer) = &self.consumer {
            // if let Some(group_id) = consumer.group_id {
                // opts.set_customer_group(CustomerGroup::new(group_id));
            // }
        }
    }

    fn fill_producer_options(&self, opts: &mut Self::ProducerOptsType) {
        todo!()
    }
}

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct ConnectOptions {
    db: u32,
    username: Option<String>,
    password: Option<String>,
    timeout: Option<Duration>,
    enable_cluster: bool,
    disable_hostname_verification: bool,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct ConsumerOptions {
    #[serde(with = "ConsumerModeRef")]
    mode: ConsumerMode,
    group_id: Option<String>,
    consumer_id: Option<String>,
    consumer_timeout: Option<Duration>,
    #[serde(with = "AutoStreamResetRef")]
    auto_stream_reset: AutoStreamReset,
    #[serde(with = "AutoCommitRef")]
    auto_commit: AutoCommit,
    auto_commit_delay: Duration,
    auto_commit_interval: Duration,
    auto_claim_interval: Option<Duration>,
    auto_claim_idle: Duration,
    batch_size: usize,
    #[serde(with = "ShardOwnershipRef")]
    shard_ownership: ShardOwnership,
    mkstream: bool,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(remote = "AutoStreamReset")]
/// Where to start streaming from if there is no priori state.
pub enum AutoStreamResetRef {
    /// Use `0` as ID, which is the earliest message.
    Earliest,
    /// Use `$` as ID, which is the latest message.
    Latest,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(remote = "AutoCommit")]
/// The auto ack / commit mechanism.
pub enum AutoCommitRef {
    /// `XREAD` with `NOACK`. This acknowledges messages as soon as they are fetched.
    /// In the event of service restart, this will likely result in messages being skipped.
    Immediate,
    /// Auto ack and commit, but only after `auto_commit_delay` has passed since messages are read.
    ///
    /// This is the default option, because users don't have to do anything extra. This also mimicks Kafka clients.
    Delayed,
    /// Do not auto ack, but continually commit acked messages to the server as new messages are read.
    /// The consumer will not commit more often than `auto_commit_interval`.
    /// You have to call [`RedisConsumer::ack`](crate::RedisConsumer::ack) manually.
    ///
    /// This is the recommended option for achieving 'at-least-once' semantics.
    Rolling,
    /// Never auto ack or commit.
    /// You have to call [`RedisConsumer::ack`](crate::RedisConsumer::ack) and [`RedisConsumer::commit`](crate::RedisConsumer::commit) manually.
    Disabled,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
#[serde(remote = "ShardOwnership")]
/// The shard ownership model.
pub enum ShardOwnershipRef {
    /// Consumers in the same group share the same shard
    Shared,
    /// Consumers claim ownership of a shard
    ///
    /// > This feature is still WIP
    Owned,
}

#[derive(Default, Debug, Clone, JsonSchema, Deserialize)]
pub struct ProducerOptions {}
