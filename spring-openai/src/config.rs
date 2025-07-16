use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Deserialize;
use spring::config::Configurable;

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
