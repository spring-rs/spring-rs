use crate::handler::{BoxedHandler, Handler};
use sea_streamer::{
    file::FileConsumerOptions, kafka::KafkaConsumerOptions, redis::RedisConsumerOptions,
    stdio::StdioConsumerOptions, ConsumerMode,
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

pub struct Consumer {
    opts: ConsumerOpts,
    handler: BoxedHandler,
}

#[derive(Default)]
pub struct ConsumerOpts {
    stream_keys: &'static [&'static str],
    mode: Option<ConsumerMode>,
    group_id: Option<String>,
    kafka_consumer_options: Option<Box<dyn FnOnce(&mut KafkaConsumerOptions) + Send + Sync>>,
    redis_consumer_options: Option<Box<dyn FnOnce(&mut RedisConsumerOptions) + Send + Sync>>,
    stdio_consumer_options: Option<Box<dyn FnOnce(&mut StdioConsumerOptions) + Send + Sync>>,
    file_consumer_options: Option<Box<dyn FnOnce(&mut FileConsumerOptions) + Send + Sync>>,
}

impl Consumer {
    pub fn consume(stream_keys: &'static [&'static str]) -> ConsumerOpts {
        ConsumerOpts {
            stream_keys,
            ..ConsumerOpts::default()
        }
    }
}

impl ConsumerOpts {
    pub fn mode(mut self, mode: ConsumerMode) -> Self {
        self.mode = Some(mode);
        self
    }
    pub fn group_id(mut self, group_id: &'static str) -> Self {
        self.group_id = Some(group_id.to_string());
        self
    }
    pub fn kafka_consumer_options<F>(mut self, fill_opts: F) -> Self
    where
        F: FnOnce(&mut KafkaConsumerOptions) + Send + Sync + 'static,
    {
        self.kafka_consumer_options = Some(Box::new(fill_opts));
        self
    }
    pub fn redis_consumer_options<F>(mut self, fill_opts: F) -> Self
    where
        F: FnOnce(&mut RedisConsumerOptions) + Send + Sync + 'static,
    {
        self.redis_consumer_options = Some(Box::new(fill_opts));
        self
    }
    pub fn stdio_consumer_options<F>(mut self, fill_opts: F) -> Self
    where
        F: FnOnce(&mut StdioConsumerOptions) + Send + Sync + 'static,
    {
        self.stdio_consumer_options = Some(Box::new(fill_opts));
        self
    }
    pub fn file_consumer_options<F>(mut self, fill_opts: F) -> Self
    where
        F: FnOnce(&mut FileConsumerOptions) + Send + Sync + 'static,
    {
        self.file_consumer_options = Some(Box::new(fill_opts));
        self
    }
    pub fn run<H, A>(self, handler: H) -> Consumer
    where
        H: Handler<A> + Sync,
        A: 'static,
    {
        Consumer {
            handler: BoxedHandler::from_handler(handler),
            opts: self,
        }
    }
}
