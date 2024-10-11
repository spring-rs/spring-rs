use std::time::Duration;

use opentelemetry_otlp::Compression;
use serde::Deserialize;
use spring::config::Configurable;

/// SMTP mailer configuration structure.
#[derive(Debug, Configurable, Clone, Deserialize)]
#[config_prefix = "opentelemetry"]
pub struct OpenTelemetryConfig {
    #[serde(flatten)]
    otel: Option<OtelExporterConfig>,
    logs: Option<OtelExporterConfig>,
    metrics: Option<OtelExporterConfig>,
    traces: Option<OtelExporterConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtelExporterConfig {
    compression: Option<Compression>,
    endpoint: Option<String>,
    headers: Option<String>,
    timeout: Option<Duration>,
}
