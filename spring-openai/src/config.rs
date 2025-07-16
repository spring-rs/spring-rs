use anyhow::anyhow;
use openai_api_rs::v1::api::OpenAIClient;
use schemars::JsonSchema;
use serde::Deserialize;
use spring::config::Configurable;
use std::collections::HashMap;

/// OpenAI Client configuration structure.
#[derive(Debug, Configurable, Clone, JsonSchema, Deserialize)]
#[config_prefix = "openai"]
pub struct OpenAIConfig {
    pub(crate) endpoint: Option<String>,
    pub(crate) api_key: Option<String>,
    #[serde(default)]
    pub(crate) headers: HashMap<String, String>,
    pub(crate) organization: Option<String>,
    pub(crate) proxy: Option<String>,
    pub(crate) timeout: Option<u64>,
}

impl OpenAIConfig {
    pub fn build(self) -> anyhow::Result<OpenAIClient> {
        let mut builder = OpenAIClient::builder();

        if let Some(endpoint) = self.endpoint {
            builder = builder.with_endpoint(endpoint);
        }
        if let Some(api_key) = self.api_key {
            builder = builder.with_api_key(api_key);
        }
        if let Some(organization) = self.organization {
            builder = builder.with_organization(organization);
        }
        if let Some(proxy) = self.proxy {
            builder = builder.with_proxy(proxy);
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_timeout(timeout);
        }
        for (key, value) in self.headers {
            builder = builder.with_header(key, value);
        }

        builder
            .build()
            .map_err(|_| anyhow!("build openai client failed"))
    }
}
