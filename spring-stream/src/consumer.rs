use std::ops::Deref;

use crate::handler::{BoxedHandler, Handler};
use sea_streamer::{
    file::FileConsumerOptions, kafka::KafkaConsumerOptions, redis::RedisConsumerOptions,
    stdio::StdioConsumerOptions, ConsumerGroup, ConsumerMode, ConsumerOptions, SeaConsumerOptions,
};

#[derive(Default)]
pub struct Consumers(Vec<Consumer>);

impl Consumers {
    pub fn add_consumer(mut self, consumer: Consumer) -> Self {
        self.0.push(consumer);
        self
    }

    pub(crate) fn merge(&mut self, consumers: Self) {
        for consumer in consumers.0 {
            self.0.push(consumer);
        }
    }
}

impl Deref for Consumers {
    type Target = Vec<Consumer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Consumer {
    pub(crate) stream_keys: &'static [&'static str],
    pub(crate) opts: ConsumerOpts,
    pub(crate) handler: BoxedHandler,
}

#[derive(Clone)]
pub struct ConsumerOpts(pub(crate) SeaConsumerOptions);

impl Consumer {
    pub fn mode(mode: ConsumerMode) -> ConsumerOpts {
        ConsumerOpts(SeaConsumerOptions::new(mode))
    }
}

impl ConsumerOpts {
    pub fn group_id(mut self, group_id: &'static str) -> Self {
        let _ = self.0.set_consumer_group(ConsumerGroup::new(group_id));
        self
    }

    pub fn kafka_consumer_options<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut KafkaConsumerOptions) -> () + Send + Sync + 'static,
    {
        self.0.set_kafka_consumer_options(func);
        self
    }

    pub fn redis_consumer_options<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut RedisConsumerOptions) -> () + Send + Sync + 'static,
    {
        self.0.set_redis_consumer_options(func);
        self
    }

    pub fn stdio_consumer_options<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut StdioConsumerOptions) -> () + Send + Sync + 'static,
    {
        self.0.set_stdio_consumer_options(func);
        self
    }

    pub fn file_consumer_options<F>(mut self, func: F) -> Self
    where
        F: FnOnce(&mut FileConsumerOptions) -> () + Send + Sync + 'static,
    {
        self.0.set_file_consumer_options(func);
        self
    }

    pub fn consume<H, A>(self, stream_keys: &'static [&'static str], handler: H) -> Consumer
    where
        H: Handler<A> + Sync,
        A: 'static,
    {
        Consumer {
            handler: BoxedHandler::from_handler(handler),
            stream_keys,
            opts: self,
        }
    }
}
