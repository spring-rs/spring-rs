//! [spring-redis](https://spring-rs.github.io/docs/plugins/spring-redis/)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

pub mod config;

use anyhow::Context;
use config::OpenTelemetryConfig;
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::{app::AppBuilder, error::Result, plugin::Plugin};
use std::time::Duration;

pub struct OpenTelemetryPlugin;

#[async_trait]
impl Plugin for OpenTelemetryPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<OpenTelemetryConfig>()
            .expect("redis plugin config load failed");
    }
}

impl OpenTelemetryPlugin {}
