mod config;
pub mod consumer;
mod handler;

use anyhow::Context;
use config::StreamConfig;
use consumer::Consumers;
pub use sea_streamer::ConsumerMode;
use sea_streamer::{
    Consumer as _, Message as _, Producer as _, SeaConsumer, SeaProducer, SeaProducerOptions,
    SeaStreamer, StreamKey, Streamer as _, StreamerUri,
};
use spring_boot::async_trait;
use spring_boot::config::Configurable;
use spring_boot::error::Result;
use spring_boot::{
    app::{App, AppBuilder},
    plugin::Plugin,
};
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

        app.add_scheduler(|app: Arc<App>| Box::new(Self::schedule(config, app)));
    }
}

impl StreamPlugin {
    async fn schedule(config: StreamConfig, app: Arc<App>) -> Result<String> {
        let streamer = Streamer::new(config, app).await?;
        let consumer = streamer.create_consumer(&[""]).await?;
        let producer = streamer.create_producer("").await?;
        // async fn process(msg: SeaMessage) -> String {
        //     msg.message().as_str()
        // }
        loop {
            let message = consumer.next().await.with_context(|| format!(""))?;
            // let message = process(message).await?;
            eprintln!("{:?}", message);
            producer
                .send(message.message())
                .with_context(|| format!(""))?; // send is non-blocking
        }
    }
}

pub struct Streamer {
    streamer: SeaStreamer,
    config: StreamConfig,
    app: Arc<App>,
}

impl Streamer {
    async fn new(config: StreamConfig, app: Arc<App>) -> Result<Self> {
        let uri = StreamerUri::from_str(config.uri.as_str())
            .with_context(|| format!("parse stream server \"{}\" failed", config.uri))?;

        let streamer = SeaStreamer::connect(uri, config.connect_options())
            .await
            .with_context(|| format!("connect stream server \"{}\" failed", config.uri))?;

        Ok(Self {
            streamer,
            config,
            app,
        })
    }

    pub async fn create_consumer(&self, stream_keys: &[&'static str]) -> Result<SeaConsumer> {
        let consumer_options = self.config.new_consumer_options(None, None);
        let consumer_options = self.config.fill_consumer_options(consumer_options);
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
        let producer_options = SeaProducerOptions::default();
        let producer_options = self.config.fill_producer_options(producer_options);

        let producer_stream_key = StreamKey::new(stream_key)
            .with_context(|| format!("producer stream key \"{}\" is valid", stream_key))?;

        Ok(self
            .streamer
            .create_producer(producer_stream_key, producer_options)
            .await
            .with_context(|| format!("create stream producer failed: {:?}", stream_key))?)
    }
}
