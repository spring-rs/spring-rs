//! [spring-redis](https://spring-rs.github.io/docs/plugins/spring-redis/)
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

pub mod config;

use anyhow::Context;
use config::OpenTelemetryConfig;
use opentelemetry::trace::{Tracer, TracerProvider};
use opentelemetry::{global, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{self as sdktrace, BatchConfig};
use opentelemetry_sdk::{resource, runtime, Resource};
use opentelemetry_semantic_conventions::attribute;
use spring::config::ConfigRegistry;
use spring::{app::AppBuilder, error::Result, plugin::Plugin};
use spring::{async_trait, log};
use std::time::Duration;
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub struct OpenTelemetryPlugin;

#[async_trait]
impl Plugin for OpenTelemetryPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<OpenTelemetryConfig>()
            .expect("redis plugin config load failed");

        let meter_provider = Self::init_metrics();
        let log_provider = Self::init_logs();
        let tracer = Self::init_tracer();

        let trace_layer = OpenTelemetryLayer::new(tracer);
        let log_layer = OpenTelemetryTracingBridge::new(&log_provider);
        let metric_layer = MetricsLayer::new(meter_provider);

        app.with_layer(trace_layer)
            .with_layer(log_layer)
            .with_layer(metric_layer)
            .add_shutdown_hook(|app| Box::new(Self::shutdown(meter_provider, log_provider)));
    }
}

impl OpenTelemetryPlugin {
    fn init_logs() -> LoggerProvider {
        opentelemetry_otlp::new_pipeline()
            .logging()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .install_batch(runtime::Tokio)
            .expect("build LogProvider failed")
    }

    fn init_metrics() -> SdkMeterProvider {
        let provider = opentelemetry_otlp::new_pipeline()
            .metrics(runtime::Tokio)
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .build()
            .expect("build MeterProvider failed");

        global::set_meter_provider(provider.clone());

        provider
    }

    fn init_tracer() -> sdktrace::Tracer {
        global::set_text_map_propagator(TraceContextPropagator::new());
        #[cfg(feature = "jaeger")]
        global::set_text_map_propagator(opentelemetry_jaeger_propagator::Propagator::new());
        #[cfg(feature = "zipkin")]
        global::set_text_map_propagator(opentelemetry_zipkin::Propagator::new());

        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(opentelemetry_otlp::new_exporter().tonic())
            .with_trace_config(sdktrace::Config::default().with_resource(Self::get_resource_attr()))
            .with_batch_config(BatchConfig::default())
            .install_batch(runtime::Tokio)
            .expect("build TraceProvider failed");

        let tracer = provider.tracer(env!("CARGO_PKG_NAME"));
        global::set_tracer_provider(provider);

        tracer
    }

    async fn shutdown(
        meter_provider: SdkMeterProvider,
        log_provider: LoggerProvider,
    ) -> Result<()> {
        global::shutdown_tracer_provider();
        meter_provider.shutdown();
        log_provider.shutdown();
        Ok(())
    }

    fn get_resource_attr() -> Resource {
        Self::app_resource().merge(&Self::infra_resource())
    }

    fn app_resource() -> Resource {
        Resource::from_schema_url(
            [
                KeyValue::new(attribute::SERVICE_NAME, env!("CARGO_PKG_NAME")),
                KeyValue::new(attribute::SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
                KeyValue::new(attribute::DEPLOYMENT_ENVIRONMENT_NAME, "develop"), // TODO: get env
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
