use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct OpenDALConfig {
    pub scheme: String,

    pub options: Option<HashMap<String, String>>,

    pub layers: Option<Vec<Layers>>,
}

#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub enum Layers {
    Chaos {
        error_ratio: f64,
    },
    Metrics,
    MimeGuess,
    Prometheus {
        requests_duration_seconds_buckets: Option<Vec<f64>>,
        bytes_total_buckets: Option<Vec<f64>>,
        path_label_level: Option<usize>,
    },
    PrometheusClient,
    Fastrace,
    Tracing,
    OtelTrace,
    Throttle {
        bandwidth: u32,
        burst: u32,
    },
    AwaitTree,
    AsyncBacktrace,
    Blocking,
    Dtrace,
}

impl Display for Layers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lf = match self {
            Layers::Chaos { .. } => "chaos",
            Layers::Metrics => "metrics",
            Layers::MimeGuess => "mimeGuess",
            Layers::Prometheus { .. } => "prometheus",
            Layers::PrometheusClient => "prometheus-client",
            Layers::Fastrace => "fastrace",
            Layers::Tracing => "tracing",
            Layers::OtelTrace => "otel-trace",
            Layers::Throttle { .. } => "throttle",
            Layers::AwaitTree => "awaitTree",
            Layers::AsyncBacktrace => "async-backtrace",
            Layers::Blocking => "blocking",
            Layers::Dtrace => "dtrace",
        };
        write!(f, "{}", lf)
    }
}