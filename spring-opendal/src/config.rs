use std::collections::HashMap;
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct OpenDALConfig {
    
    pub scheme: String,
    
    pub options: Option<HashMap<String, String>>
    
    // todo: batch retry
    // retry: Option<RetryConfig>,
    // RetryConfig { timeout: u64 }
    // bala bala
}