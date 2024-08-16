mod config;

use std::sync::OnceLock;

use crate::{app::AppBuilder, config::Configurable};
use anyhow::Context;
use config::{Format, LogLevel, LoggerConfig, Rotation};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_log::LogTracer;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    fmt::{self, MakeWriter},
    Layer, Registry,
};

pub(crate) struct LogPlugin;

impl Configurable for LogPlugin {
    fn config_prefix(&self) -> &str {
        "logger"
    }
}

impl LogPlugin {
    pub(crate) fn build(&self, app: &mut AppBuilder) {
        LogTracer::init().context("init tracing_log faile").unwrap();

        let registry = std::mem::take(&mut app.tracing_registry);

        let config = app
            .get_config::<LoggerConfig>(self)
            .context("tracing plugin config load failed")
            .unwrap();

        let layers = build_logger_layers(config);

        if !layers.is_empty() {
            registry.with(layers).init();
        }
    }
}

// Keep nonblocking file appender work guard
static NONBLOCKING_WORK_GUARD_KEEP: OnceLock<WorkerGuard> = OnceLock::new();

fn build_logger_layers(config: LoggerConfig) -> Vec<Box<dyn Layer<Registry> + Sync + Send>> {
    let mut layers = Vec::new();

    if let Some(file_appender_config) = config.file_appender.as_ref() {
        if file_appender_config.enable {
            let mut rolling_builder = tracing_appender::rolling::Builder::default()
                .max_log_files(file_appender_config.max_log_files)
                .filename_prefix(&file_appender_config.filename_prefix)
                .filename_suffix(&file_appender_config.filename_suffix);

            rolling_builder = match file_appender_config.rotation {
                Rotation::Minutely => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::MINUTELY)
                }
                Rotation::Hourly => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::HOURLY)
                }
                Rotation::Daily => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::DAILY)
                }
                Rotation::Never => {
                    rolling_builder.rotation(tracing_appender::rolling::Rotation::NEVER)
                }
            };

            let file_appender = rolling_builder
                .build(&file_appender_config.dir)
                .expect("logger file appender initialization failed");

            let file_appender_layer = if file_appender_config.non_blocking {
                let (non_blocking_file_appender, work_guard) =
                    tracing_appender::non_blocking(file_appender);
                NONBLOCKING_WORK_GUARD_KEEP.set(work_guard).unwrap();
                init_fmt_layer(non_blocking_file_appender, &config.format, false)
            } else {
                init_fmt_layer(file_appender, &config.format, false)
            };
            layers.push(file_appender_layer);
        }
    }

    if config.enable {
        let stdout_layer = init_fmt_layer(std::io::stdout, &config.format, true);
        layers.push(stdout_layer);
    }

    return layers;
}

fn init_fmt_layer<W2>(
    make_writer: W2,
    format: &Format,
    ansi: bool,
) -> Box<dyn Layer<Registry> + Sync + Send>
where
    W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
{
    match format {
        Format::Compact => fmt::Layer::default()
            .with_ansi(ansi)
            .with_writer(make_writer)
            .compact()
            .boxed(),
        Format::Pretty => fmt::Layer::default()
            .with_ansi(ansi)
            .with_writer(make_writer)
            .pretty()
            .boxed(),
        Format::Json => fmt::Layer::default()
            .with_ansi(ansi)
            .with_writer(make_writer)
            .json()
            .boxed(),
    }
}

fn init_env_filter(override_filter: Option<&String>, level: &LogLevel) -> EnvFilter {
    EnvFilter::try_from_default_env().context("").unwrap()
}
