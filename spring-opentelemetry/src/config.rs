use std::{str::FromStr, time::Duration};

use http::{HeaderMap, HeaderName, HeaderValue};
use opentelemetry_otlp::{Compression, TonicExporterBuilder, WithExportConfig};
use serde::Deserialize;
use spring::config::Configurable;
use tonic::metadata::MetadataMap;

/// SMTP mailer configuration structure.
#[derive(Debug, Configurable, Clone, Deserialize)]
#[config_prefix = "opentelemetry"]
pub struct OpenTelemetryConfig {
    #[serde(flatten)]
    pub(crate) otel: Option<OtelExporterConfig>,
    pub(crate) logs: Option<OtelExporterConfig>,
    pub(crate) metrics: Option<OtelExporterConfig>,
    pub(crate) traces: Option<OtelExporterConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OtelExporterConfig {
    pub(crate) compression: Option<Compression>,
    pub(crate) endpoint: Option<String>,
    pub(crate) headers: Option<String>,
    pub(crate) timeout: Option<Duration>,
}

pub(crate) trait Merger {
    fn merge(self, other: Self) -> Self;
}

impl Merger for Option<OtelExporterConfig> {
    fn merge(self, other: Self) -> Self {
        let left = self?;
        let right = other?;
        Some(OtelExporterConfig {
            compression: left.compression.or(right.compression),
            endpoint: left.endpoint.or(right.endpoint),
            headers: left.headers.or(right.headers),
            timeout: left.timeout.or(right.timeout),
        })
    }
}

impl OtelExporterConfig {
    pub(crate) fn apply_config(self, mut builder: TonicExporterBuilder) -> TonicExporterBuilder {
        if let Some(compression) = self.compression {
            builder = builder.with_compression(compression);
        }
        if let Some(endpoint) = self.endpoint {
            builder = builder.with_endpoint(endpoint);
        }
        if let Some(timeout) = self.timeout {
            builder = builder.with_timeout(timeout);
        }
        if let Some(headers) = self.headers {
            let headers = Self::parse_header_string(&headers)
                .filter_map(|(key, value)| {
                    Some((
                        HeaderName::from_str(key).ok()?,
                        HeaderValue::from_str(&value).ok()?,
                    ))
                })
                .collect::<HeaderMap>();
            builder = builder.with_metadata(MetadataMap::from_headers(headers));
        }
        builder
    }
    fn parse_header_string(value: &str) -> impl Iterator<Item = (&str, String)> {
        value
            .split_terminator(',')
            .map(str::trim)
            .filter_map(Self::parse_header_key_value_string)
    }

    fn parse_header_key_value_string(key_value_string: &str) -> Option<(&str, String)> {
        key_value_string
            .split_once('=')
            .map(|(key, value)| {
                (
                    key.trim(),
                    Self::url_decode(value.trim()).unwrap_or(value.to_string()),
                )
            })
            .filter(|(key, value)| !key.is_empty() && !value.is_empty())
    }

    fn url_decode(value: &str) -> Option<String> {
        let mut result = String::with_capacity(value.len());
        let mut chars_to_decode = Vec::<u8>::new();
        let mut all_chars = value.chars();

        loop {
            let ch = all_chars.next();

            if ch.is_some() && ch.unwrap() == '%' {
                chars_to_decode.push(
                    u8::from_str_radix(&format!("{}{}", all_chars.next()?, all_chars.next()?), 16)
                        .ok()?,
                );
                continue;
            }

            if !chars_to_decode.is_empty() {
                result.push_str(std::str::from_utf8(&chars_to_decode).ok()?);
                chars_to_decode.clear();
            }

            if let Some(c) = ch {
                result.push(c);
            } else {
                return Some(result);
            }
        }
    }
}
