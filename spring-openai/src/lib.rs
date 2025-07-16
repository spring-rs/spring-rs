//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io/docs/plugins/spring-openai)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

pub mod config;

pub use openai_api_rs::*;

use crate::config::OpenAIConfig;
use openai_api_rs::v1::api::OpenAIClient as OriginOpenAIClient;
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::plugin::MutableComponentRegistry;
use spring::{app::AppBuilder, plugin::Plugin};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

pub struct OpenAIPlugin;

#[async_trait]
impl Plugin for OpenAIPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<OpenAIConfig>()
            .expect("openai plugin config load failed");

        let mut builder = OriginOpenAIClient::builder();

        if let Some(endpoint) = config.endpoint {
            builder = builder.with_endpoint(endpoint);
        }
        if let Some(api_key) = config.api_key {
            builder = builder.with_api_key(api_key);
        }
        if let Some(organization) = config.organization {
            builder = builder.with_organization(organization);
        }
        if let Some(proxy) = config.proxy {
            builder = builder.with_proxy(proxy);
        }
        if let Some(timeout) = config.timeout {
            builder = builder.with_timeout(timeout);
        }
        for (key, value) in config.headers {
            builder = builder.with_header(key, value);
        }

        let openai_client = builder.build().expect("openai client build failed");

        app.add_component(OpenAIClient(Arc::new(openai_client)));
    }
}

#[derive(Debug, Clone)]
pub struct OpenAIClient(Arc<OriginOpenAIClient>);

impl Deref for OpenAIClient {
    type Target = OriginOpenAIClient;

    fn deref(&self) -> &OriginOpenAIClient {
        &*self.0
    }
}

impl DerefMut for OpenAIClient {
    fn deref_mut(&mut self) -> &mut OriginOpenAIClient {
        &mut *self.0.clone()
    }
}
