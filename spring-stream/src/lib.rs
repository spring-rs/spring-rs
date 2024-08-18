mod config;

use anyhow::Context;
use spring_boot::async_trait;
use config::StreamConfig;
use sea_streamer::{
    Consumer, Message, Producer, SeaMessage, SeaProducerOptions, SeaStreamer, StreamKey, Streamer,
    StreamerUri,
};
use spring_boot::config::Configurable;
use spring_boot::error::Result;
use spring_boot::{
    app::{App, AppBuilder},
    plugin::Plugin,
};
use std::{str::FromStr, sync::Arc};

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
        let uri = StreamerUri::from_str(config.uri.as_str())
            .with_context(|| format!("parse stream server \"{}\" failed", config.uri))?;

        let streamer = SeaStreamer::connect(uri, config.connect_options())
            .await
            .with_context(|| format!("connect stream server \"{}\" failed", config.uri))?;

        let consumer_options = config.new_consumer_options(None, None);
        let producer_options = SeaProducerOptions::default();
        let consumer_options = config.fill_consumer_options(consumer_options);
        let producer_options = config.fill_producer_options(producer_options);
        let consumer_stream_key = "";
        let consumer_stream_key = StreamKey::new(consumer_stream_key)
            .with_context(|| format!("stream key \"{}\" is valid", consumer_stream_key))?;
        let producer_stream_key = "";
        let producer_stream_key = StreamKey::new(producer_stream_key)
            .with_context(|| format!("stream key \"{}\" is valid", producer_stream_key))?;

        let consumer = streamer
            .create_consumer(&[consumer_stream_key], consumer_options)
            .await
            .with_context(|| format!(""))?;
        let producer = streamer
            .create_producer(producer_stream_key, producer_options)
            .await
            .with_context(|| format!(""))?;

        loop {
            let message: SeaMessage = consumer.next().await.with_context(|| format!(""))?;
            // let message = process(message).await?;
            // eprintln!("{message}");
            producer
                .send(message.message())
                .with_context(|| format!(""))?; // send is non-blocking
        }
    }
}
