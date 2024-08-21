pub mod config;

use crate::config::*;
use anyhow::Result;
use opendal::Operator;
use spring_boot::app::AppBuilder;
use spring_boot::async_trait;
use spring_boot::config::Configurable;
use spring_boot::plugin::Plugin;
use std::str::FromStr;

pub type Op = Operator;

#[derive(Configurable)]
#[config_prefix = "opendal"]
pub struct OpenDALPlugin;

#[async_trait]
impl Plugin for OpenDALPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<OpenDALConfig>(self)
            .expect("OpenDAL plugin config load failed");

        let connect: Operator = Self::operator(config).expect("OpenDAL operator construct failed");
        app.add_component(connect);
    }
}

impl OpenDALPlugin {
    pub fn operator(config: OpenDALConfig) -> Result<Operator> {
        let scheme = opendal::Scheme::from_str(&config.scheme).map_err(|err| {
            opendal::Error::new(opendal::ErrorKind::Unexpected, "not supported scheme")
                .set_source(err)
        })?;

        let options = config.options.unwrap_or_default();

        #[allow(unused_mut)]
        let mut op = Operator::via_iter(scheme, options)?;

        if let Some(layers) = config.layers {
            for layer in layers {
                log::debug!("layer-{} enable", layer);
                match layer {
                    #[cfg(feature = "layers-chaos")]
                    Layers::Chaos { error_ratio } => {
                        op = op.layer(opendal::layers::ChaosLayer::new(error_ratio));
                    }
                    #[cfg(feature = "layers-metrics")]
                    Layers::Metrics => {
                        op = op.layer(opendal::layers::MetricsLayer);
                    }
                    #[cfg(feature = "layers-mime-guess")]
                    Layers::MimeGuess => {
                        op = op.layer(opendal::layers::MimeGuessLayer::default());
                    }
                    #[cfg(feature = "layers-prometheus")]
                    Layers::Prometheus {
                        requests_duration_seconds_buckets,
                        bytes_total_buckets,
                        path_label_level,
                    } => {
                        let registry = prometheus::default_registry();
                        let mut prometheus_layer =
                            opendal::layers::PrometheusLayer::with_registry(registry.clone());
                        if let Some(requests_duration_seconds_buckets) =
                            requests_duration_seconds_buckets
                        {
                            prometheus_layer = prometheus_layer.requests_duration_seconds_buckets(
                                requests_duration_seconds_buckets,
                            );
                        }
                        if let Some(bytes_total_buckets) = bytes_total_buckets {
                            prometheus_layer =
                                prometheus_layer.bytes_total_buckets(bytes_total_buckets);
                        }
                        if let Some(path_label_level) = path_label_level {
                            prometheus_layer = prometheus_layer.enable_path_label(path_label_level);
                        }
                        op = op.layer(prometheus_layer);
                    }
                    #[cfg(feature = "layers-prometheus-client")]
                    Layers::PrometheusClient => {
                        let mut registry = prometheus_client::registry::Registry::default();
                        op = op.layer(opendal::layers::PrometheusClientLayer::new(&mut registry));
                    }
                    #[cfg(feature = "layers-fastrace")]
                    Layers::Fastrace => {
                        op = op.layer(opendal::layers::FastraceLayer);
                    }
                    #[cfg(feature = "layers-tracing")]
                    Layers::Tracing => {
                        op = op.layer(opendal::layers::TracingLayer);
                    }
                    #[cfg(feature = "layers-otel-trace")]
                    Layers::OtelTrace => {
                        op = op.layer(opendal::layers::OtelTraceLayer);
                    }
                    #[cfg(feature = "layers-throttle")]
                    Layers::Throttle { bandwidth, burst } => {
                        op = op.layer(opendal::layers::ThrottleLayer::new(bandwidth, burst));
                    }
                    #[cfg(feature = "layers-await-tree")]
                    Layers::AwaitTree => {
                        op = op.layer(opendal::layers::AwaitTreeLayer::new());
                    }
                    #[cfg(feature = "layers-async-backtrace")]
                    Layers::AsyncBacktrace => {
                        op = op.layer(opendal::layers::AsyncBacktraceLayer);
                    }
                    #[cfg(feature = "layers-blocking")]
                    Layers::Blocking => {
                        if !cfg!(feature = "test-layers") && op.info().native_capability().blocking {
                            log::warn!("Blocking layer is not necessary for this operator");
                            continue;
                        }
                        match tokio::runtime::Handle::try_current() {
                            Ok(handle) => {
                                let _guard = handle.enter();
                                op = op.layer(opendal::layers::BlockingLayer::create()?);
                            }
                            Err(e) => {
                                log::error!("{}", e);
                            }
                        }
                    }
                    #[cfg(all(target_os = "linux", feature = "layers-dtrace"))]
                    Layers::Dtrace => {
                        op = op.layer(opendal::layers::DtraceLayer::default());
                    }
                    #[allow(unreachable_patterns)]
                    _ => {
                        panic!("Maybe you forgotten to enable the [services-{}] feature!", layer);
                    }
                }
            }
        }

        Ok(op)
    }
}
