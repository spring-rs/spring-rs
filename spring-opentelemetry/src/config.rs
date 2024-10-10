use schemars::JsonSchema;
use serde::Deserialize;
use spring::config::Configurable;

/// SMTP mailer configuration structure.
#[derive(Debug, Configurable, Clone, JsonSchema, Deserialize)]
#[config_prefix = "opentelemetry"]
pub struct OpenTelemetryConfig {}
