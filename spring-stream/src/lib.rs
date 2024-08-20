mod config;
pub mod consumer;
mod extractor;
mod handler;

use anyhow::Context;
use config::StreamConfig;
use consumer::{Consumer, ConsumerOpts, Consumers};
use handler::BoxedHandler;
pub use sea_streamer::ConsumerMode;
use sea_streamer::{
    Consumer as _, SeaConsumer, SeaProducer, SeaStreamer, StreamKey, Streamer as _, StreamerUri,
};
use spring_boot::async_trait;
use spring_boot::config::Configurable;
use spring_boot::error::Result;
use spring_boot::{
    app::{App, AppBuilder},
    plugin::Plugin,
};
use std::ops::Deref;
use std::{str::FromStr, sync::Arc};

pub trait StreamConfigurator {
    fn add_consumer(&mut self, consumers: Consumers) -> &mut Self;
}

impl StreamConfigurator for AppBuilder {
    fn add_consumer(&mut self, new_consumers: Consumers) -> &mut Self {
        if let Some(consumers) = self.get_component::<Consumers>() {
            unsafe {
                let raw_ptr = Arc::into_raw(consumers);
                let consumers = &mut *(raw_ptr as *mut Consumers);
                consumers.merge(new_consumers);
            }
            self
        } else {
            self.add_component(new_consumers)
        }
    }
}

#[derive(Configurable)]
#[config_prefix = "stream"]
pub struct StreamPlugin;

#[async_trait]
impl Plugin for StreamPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<StreamConfig>(self)
            .expect("sea-streamer plugin config load failed");

        let streamer = Streamer::new(config).await.expect("create streamer failed");

        if let Some(consumers) = app.get_component::<Consumers>() {
            for Consumer {
                stream_keys,
                opts,
                handler,
            } in consumers.deref().iter()
            {
                let consumer = streamer
                    .create_consumer(stream_keys, opts.clone())
                    .await
                    .expect("create stream consumer faile");
                let handler = handler.clone();
                app.add_scheduler(|app: Arc<App>| Box::new(Self::schedule(consumer, handler, app)));
            }
        } else {
            tracing::info!("not consumer be registry");
        }
        app.add_component(streamer);
    }
}

impl StreamPlugin {
    async fn schedule(
        consumer: SeaConsumer,
        handler: BoxedHandler,
        app: Arc<App>,
    ) -> Result<String> {
        loop {
            let message = consumer.next().await.with_context(|| format!(""))?;
            handler.call(message, app.clone()).await;
        }
    }
}

pub struct Streamer {
    streamer: SeaStreamer,
    config: StreamConfig,
}

impl Streamer {
    async fn new(config: StreamConfig) -> Result<Self> {
        let uri = StreamerUri::from_str(config.uri.as_str())
            .with_context(|| format!("parse stream server \"{}\" failed", config.uri))?;

        let streamer = SeaStreamer::connect(uri, config.connect_options())
            .await
            .with_context(|| format!("connect stream server \"{}\" failed", config.uri))?;

        Ok(Self { streamer, config })
    }

    pub async fn create_consumer(
        &self,
        stream_keys: &'static [&'static str],
        opts: ConsumerOpts,
    ) -> Result<SeaConsumer> {
        let consumer_options = self.config.new_consumer_options(opts);
        let mut consumer_stream_keys = Vec::with_capacity(stream_keys.len());
        for key in stream_keys {
            consumer_stream_keys.push(
                StreamKey::new(*key)
                    .with_context(|| format!("consumer stream key \"{}\" is valid", key))?,
            );
        }
        Ok(self
            .streamer
            .create_consumer(&consumer_stream_keys, consumer_options)
            .await
            .with_context(|| format!("create stream consumer failed: {:?}", stream_keys))?)
    }

    pub fn send() {
        todo!()
    }

    async fn create_producer(&self, stream_key: &'static str) -> Result<SeaProducer> {
        let producer_options = self.config.new_producer_options();

        let producer_stream_key = StreamKey::new(stream_key)
            .with_context(|| format!("producer stream key \"{}\" is valid", stream_key))?;

        Ok(self
            .streamer
            .create_producer(producer_stream_key, producer_options)
            .await
            .with_context(|| format!("create stream producer failed: {:?}", stream_key))?)
    }
}
