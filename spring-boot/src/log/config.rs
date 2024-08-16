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
    #[serde(default = "default_dir")]
    pub dir: String,
    #[serde(default = "default_prefix")]
    pub filename_prefix: String,
    #[serde(default = "default_suffix")]
    pub filename_suffix: String,
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

fn default_dir() -> String {
    "./logs".to_string()
}

fn default_prefix() -> String {
    "app".to_string()
}

fn default_suffix() -> String {
    ".log".to_string()
}
