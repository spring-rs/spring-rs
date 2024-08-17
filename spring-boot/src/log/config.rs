use std::fmt::Display;

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub(crate) struct LoggerConfig {
    pub enable: bool,

    /// Enable nice display of backtraces, in development this should be on.
    /// Turn it off in performance sensitive production deployments.
    #[serde(default)]
    pub pretty_backtrace: bool,

    /// Set the logger level.
    ///
    /// * options: `trace` | `debug` | `info` | `warn` | `error`
    pub level: LogLevel,

    /// Set the logger format.
    ///
    /// * options: `compact` | `pretty` | `json`
    pub format: Format,

    /// Override our custom tracing filter.
    ///
    /// Set this to your own filter if you want to see traces from internal
    /// libraries. See more [here](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html#directives)
    pub override_filter: Option<String>,

    /// Set this if you want to write log to file
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

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Off => "off",
                Self::Trace => "trace",
                Self::Debug => "debug",
                Self::Info => "debug",
                Self::Warn => "warn",
                Self::Error => "error",
            }
        )
    }
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

impl Into<tracing_appender::rolling::Rotation> for Rotation {
    fn into(self) -> tracing_appender::rolling::Rotation {
        match self {
            Self::Minutely => tracing_appender::rolling::Rotation::MINUTELY,
            Self::Hourly => tracing_appender::rolling::Rotation::HOURLY,
            Self::Daily => tracing_appender::rolling::Rotation::DAILY,
            Self::Never => tracing_appender::rolling::Rotation::NEVER,
        }
    }
}
