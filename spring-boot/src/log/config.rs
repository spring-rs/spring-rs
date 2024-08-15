use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub(crate) struct LoggerConfig {
    pub enable: bool,
    pub pretty_backtrace: bool,
    pub level: LogLevel,
    pub format: Format,
    pub override_filter: Option<String>,
    pub file_appender: Option<LoggerFileAppender>,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
pub(crate) enum LogLevel {
    /// The "off" level.
    #[serde(rename = "off")]
    Off,
    /// The "trace" level.
    #[serde(rename = "trace")]
    Trace,
    /// The "debug" level.
    #[serde(rename = "debug")]
    Debug,
    /// The "info" level.
    #[serde(rename = "info")]
    #[default]
    Info,
    /// The "warn" level.
    #[serde(rename = "warn")]
    Warn,
    /// The "error" level.
    #[serde(rename = "error")]
    Error,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
pub(crate) enum Format {
    #[serde(rename = "compact")]
    #[default]
    Compact,
    #[serde(rename = "pretty")]
    Pretty,
    #[serde(rename = "json")]
    Json,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub(crate) struct LoggerFileAppender {
    pub enable: bool,
    pub non_blocking: bool,
    pub level: LogLevel,
    pub format: Format,
    pub rotation: Rotation,
    pub dir: Option<String>,
    pub filename_prefix: Option<String>,
    pub filename_suffix: Option<String>,
    pub max_log_files: usize,
}

#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
pub(crate) enum Rotation {
    #[serde(rename = "minutely")]
    Minutely,
    #[serde(rename = "hourly")]
    #[default]
    Hourly,
    #[serde(rename = "daily")]
    Daily,
    #[serde(rename = "never")]
    Never,
}
