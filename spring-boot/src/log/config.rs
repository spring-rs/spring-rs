use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub(crate) struct TracingConfig {}
