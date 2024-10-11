//! [spring-opentelemetry](https://spring-rs.github.io/docs/plugins/spring-opentelemetry/)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

pub mod config;

#[rustfmt::skip]
pub use opentelemetry_otlp::{
    OTEL_EXPORTER_OTLP_COMPRESSION,
    OTEL_EXPORTER_OTLP_ENDPOINT,
    OTEL_EXPORTER_OTLP_HEADERS,
    OTEL_EXPORTER_OTLP_TIMEOUT,
    // logs
    OTEL_EXPORTER_OTLP_LOGS_COMPRESSION,
    OTEL_EXPORTER_OTLP_LOGS_ENDPOINT,
    OTEL_EXPORTER_OTLP_LOGS_HEADERS,
    OTEL_EXPORTER_OTLP_LOGS_TIMEOUT,
    // metrics
    OTEL_EXPORTER_OTLP_METRICS_COMPRESSION,
    OTEL_EXPORTER_OTLP_METRICS_ENDPOINT,
    OTEL_EXPORTER_OTLP_METRICS_HEADERS,
    OTEL_EXPORTER_OTLP_METRICS_TIMEOUT,
    // trace
    OTEL_EXPORTER_OTLP_TRACES_COMPRESSION,
    OTEL_EXPORTER_OTLP_TRACES_ENDPOINT,
    OTEL_EXPORTER_OTLP_TRACES_HEADERS,
    OTEL_EXPORTER_OTLP_TRACES_TIMEOUT,
};

use anyhow::Context;
use config::{Merger, OpenTelemetryConfig};
use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{self as sdktrace, BatchConfig};
use opentelemetry_sdk::{resource, runtime, Resource};
use opentelemetry_semantic_conventions::attribute;
use spring::async_trait;
use spring::config::env::Env;
use spring::config::ConfigRegistry;
use spring::log::LayersReloader;
use spring::{app::AppBuilder, error::Result, plugin::Plugin};
use std::time::Duration;
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};

pub struct OpenTelemetryPlugin;

#[async_trait]
impl Plugin for OpenTelemetryPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<OpenTelemetryConfig>()
            .expect("redis plugin config load failed");

        let env = app.get_env().clone();
        let meter_provider = config.init_metrics(env);
        let log_provider = config.init_logs(env);
        let tracer = config.init_tracer(env);

        let trace_layer = OpenTelemetryLayer::new(tracer);
        let log_layer = OpenTelemetryTracingBridge::new(&log_provider);
        let metric_layer = MetricsLayer::new(meter_provider.clone());

        let reloader_ref = app
            .get_component_ref::<LayersReloader>()
            .expect("get layers reloader failed");
        reloader_ref
            .modify(|layers| {
                layers.push(Box::new(trace_layer));
                layers.push(Box::new(log_layer));
                layers.push(Box::new(metric_layer));
            })
            .expect("reload layers for opentelemetry failed");
        app.add_shutdown_hook(move |_| Box::new(Self::shutdown(meter_provider, log_provider)));
    }
}

impl OpenTelemetryConfig {
    fn init_logs(&self, env: Env) -> LoggerProvider {
        let Self { otel, logs, .. } = self;
        let exporter_config = logs.clone().merge(otel.clone());
        let exporter = match exporter_config {
            None => opentelemetry_otlp::new_exporter().tonic(),
            Some(c) => c.apply_config(opentelemetry_otlp::new_exporter().tonic()),
        };
        opentelemetry_otlp::new_pipeline()
            .logging()
            .with_exporter(exporter)
            .with_resource(Self::get_resource_attr(env))
            .install_batch(runtime::Tokio)
            .expect("build LogProvider failed")
    }

    fn init_metrics(&self, env: Env) -> SdkMeterProvider {
        let Self { otel, metrics, .. } = self;
        let exporter_config = metrics.clone().merge(otel.clone());
        let exporter = match exporter_config {
            None => opentelemetry_otlp::new_exporter().tonic(),
            Some(c) => c.apply_config(opentelemetry_otlp::new_exporter().tonic()),
        };
        let provider = opentelemetry_otlp::new_pipeline()
            .metrics(runtime::Tokio)
            .with_exporter(exporter)
            .with_resource(Self::get_resource_attr(env))
            .build()
            .expect("build MeterProvider failed");

        global::set_meter_provider(provider.clone());
        tracing::debug!("metrics provider installed");

        provider
    }

    fn init_tracer(&self, env: Env) -> sdktrace::Tracer {
        let Self { otel, traces, .. } = self;
        global::set_text_map_propagator(TraceContextPropagator::new());
        #[cfg(feature = "jaeger")]
        global::set_text_map_propagator(opentelemetry_jaeger_propagator::Propagator::new());
        #[cfg(feature = "zipkin")]
        global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());

        let exporter_config = traces.clone().merge(otel.clone());
        let exporter = match exporter_config {
            None => opentelemetry_otlp::new_exporter().tonic(),
            Some(c) => c.apply_config(opentelemetry_otlp::new_exporter().tonic()),
        };
        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(exporter)
            .with_trace_config(
                sdktrace::Config::default().with_resource(Self::get_resource_attr(env)),
            )
            .with_batch_config(BatchConfig::default())
            .install_batch(runtime::Tokio)
            .expect("build TraceProvider failed");

        let tracer = provider.tracer(env!("CARGO_PKG_NAME"));
        global::set_tracer_provider(provider);
        tracing::debug!("tracer provider installed");

        tracer
    }
    fn get_resource_attr(env: Env) -> Resource {
        Self::app_resource(env).merge(&Self::infra_resource())
    }

    fn app_resource(env: Env) -> Resource {
        Resource::from_schema_url(
            [
                KeyValue::new(attribute::SERVICE_NAME, env!("CARGO_PKG_NAME")),
                KeyValue::new(attribute::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(attribute::DEPLOYMENT_ENVIRONMENT_NAME, format!("{:?}", env)),
            ],
            opentelemetry_semantic_conventions::SCHEMA_URL,
        )
    }

    fn infra_resource() -> Resource {
        Resource::from_detectors(
            Duration::from_secs(0),
            vec![
                #[cfg(feature = "more-resource")]
                Box::new(opentelemetry_resource_detectors::HostResourceDetector::default()),
                #[cfg(feature = "more-resource")]
                Box::new(opentelemetry_resource_detectors::OsResourceDetector),
                #[cfg(feature = "more-resource")]
                Box::new(opentelemetry_resource_detectors::ProcessResourceDetector),
                Box::new(resource::SdkProvidedResourceDetector),
                Box::new(resource::TelemetryResourceDetector),
                Box::new(resource::EnvResourceDetector::new()),
            ],
        )
    }
}

impl OpenTelemetryPlugin {
    async fn shutdown(
        meter_provider: SdkMeterProvider,
        log_provider: LoggerProvider,
    ) -> Result<String> {
        global::shutdown_tracer_provider();
        meter_provider
            .shutdown()
            .context("shutdown meter provider failed")?;
        log_provider
            .shutdown()
            .context("shutdown log provider failed")?;
        Ok("OpenTelemetry shutdown successful".into())
    }
}
