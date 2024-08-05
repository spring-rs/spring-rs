use std::str::FromStr;

use anyhow::Context;
use async_trait::async_trait;
use autumn_boot::{app::AppBuilder, plugin::Plugin};
use config::StreamConfig;
use sea_streamer::{
    file::FileStreamer, kafka::KafkaStreamer, redis::RedisStreamer, stdio::StdioStreamer,
    SeaStreamer, Streamer, StreamerUri,
};

mod config;

pub struct StreamerPlugin;

#[async_trait]
impl Plugin for StreamerPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<StreamConfig>(self)
            .context(format!("sea-streamer plugin config load failed"))
            .unwrap();

        let uri = StreamerUri::from_str(config.uri.as_str())
            .with_context(|| format!("parse stream server {} failed", config.uri))
            .unwrap();

        let streamer = SeaStreamer::connect(uri, config.connect_options())
            .await
            .with_context(|| format!("connect stream server {} failed", config.uri))
            .unwrap();

        
    }

    fn config_prefix(&self) -> &str {
        "stream"
    }
}
