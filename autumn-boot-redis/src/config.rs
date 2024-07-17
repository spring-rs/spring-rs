use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct RedisConfig {
    /// The URI for connecting to the Redis server. For example:
    /// <redis://127.0.0.1/>
    pub uri: String,
}
