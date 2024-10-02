mod config;

use crate::app::AppBuilder;
use crate::config::ConfigRegistry;
use config::{Format, LogLevel, LoggerConfig, TimeStyle, WithFields};
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::time::{ChronoLocal, ChronoUtc, FormatTime, SystemTime, Uptime};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{
    fmt::{self, MakeWriter},
    Layer, Registry,
};

pub(crate) struct LogPlugin;

impl LogPlugin {
    pub(crate) fn build(&self, app: &mut AppBuilder) {
        let registry = std::mem::take(&mut app.tracing_registry);

        let config = app
            .get_config::<LoggerConfig>()
            .expect("tracing plugin config load failed");

        if config.pretty_backtrace {
            std::env::set_var("RUST_BACKTRACE", "1");
            log::warn!(
                "pretty backtraces are enabled (this is great for development but has a runtime cost for production. disable with `logger.pretty_backtrace` in your config)"
            );
        }

        let layers = build_logger_layers(&config);

        if !layers.is_empty() {
            let env_filter = init_env_filter(config.override_filter, &config.level);
            registry.with(layers).with(env_filter).init();
        }
    }
}

// Keep nonblocking file appender work guard
static NONBLOCKING_WORK_GUARD_KEEP: OnceLock<WorkerGuard> = OnceLock::new();

fn build_logger_layers(config: &LoggerConfig) -> Vec<Box<dyn Layer<Registry> + Sync + Send>> {
    let mut layers = Vec::new();

    if let Some(file_config) = &config.file_appender {
        if file_config.enable {
            let file_appender = tracing_appender::rolling::Builder::default()
                .max_log_files(file_config.max_log_files)
                .filename_prefix(&file_config.filename_prefix)
                .filename_suffix(&file_config.filename_suffix)
                .rotation(file_config.rotation.clone().into())
                .build(&file_config.dir)
                .expect("logger file appender initialization failed");

            let file_appender_layer = if file_config.non_blocking {
                let (non_blocking_file_appender, work_guard) =
                    tracing_appender::non_blocking(file_appender);
                NONBLOCKING_WORK_GUARD_KEEP.set(work_guard).unwrap();
                build_fmt_layer(
                    non_blocking_file_appender,
                    &file_config.format,
                    config,
                    false,
                )
            } else {
                build_fmt_layer(file_appender, &file_config.format, config, false)
            };
            layers.push(file_appender_layer);
        }
    }

    if config.enable {
        layers.push(build_fmt_layer(
            std::io::stdout,
            &config.format,
            config,
            true,
        ));
    }

    layers
}

fn build_fmt_layer<W2>(
    make_writer: W2,
    format: &Format,
    config: &LoggerConfig,
    ansi: bool,
) -> Box<dyn Layer<Registry> + Sync + Send>
where
    W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
{
    let LoggerConfig {
        time_style,
        time_pattern,
        with_fields,
        ..
    } = config;
    match time_style {
        TimeStyle::SystemTime => {
            config_layer_with_timer(make_writer, format, SystemTime, ansi, with_fields)
        }
        TimeStyle::Uptime => {
            config_layer_with_timer(make_writer, format, Uptime::default(), ansi, with_fields)
        }
        TimeStyle::ChronoLocal => config_layer_with_timer(
            make_writer,
            format,
            ChronoLocal::new(time_pattern.to_string()),
            ansi,
            with_fields,
        ),
        TimeStyle::ChronoUtc => config_layer_with_timer(
            make_writer,
            format,
            ChronoUtc::new(time_pattern.to_string()),
            ansi,
            with_fields,
        ),
        TimeStyle::None => config_layer_without_time(make_writer, format, ansi, with_fields),
    }
}

fn config_layer_with_timer<W2, T>(
    make_writer: W2,
    format: &Format,
    timer: T,
    ansi: bool,
    with_fields: &Vec<WithFields>,
) -> Box<dyn Layer<Registry> + Sync + Send>
where
    W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
    T: FormatTime + Sync + Send + 'static,
{
    let mut layer = fmt::Layer::default()
        .with_ansi(ansi)
        .with_writer(make_writer)
        .with_timer(timer);

    for field in with_fields {
        match field {
            WithFields::File => layer = layer.with_file(true),
            WithFields::LineNumber => layer = layer.with_line_number(true),
            WithFields::ThreadId => layer = layer.with_thread_ids(true),
            WithFields::ThreadName => layer = layer.with_thread_names(true),
            WithFields::InternalErrors => layer = layer.log_internal_errors(true),
        }
    }

    match format {
        Format::Compact => layer.compact().boxed(),
        Format::Pretty => layer.pretty().boxed(),
        Format::Json => layer.json().boxed(),
    }
}

fn config_layer_without_time<W2>(
    make_writer: W2,
    format: &Format,
    ansi: bool,
    with_fields: &Vec<WithFields>,
) -> Box<dyn Layer<Registry> + Sync + Send>
where
    W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
{
    let mut layer = fmt::Layer::default()
        .with_ansi(ansi)
        .with_writer(make_writer)
        .without_time();

    for field in with_fields {
        match field {
            WithFields::File => layer = layer.with_file(true),
            WithFields::LineNumber => layer = layer.with_line_number(true),
            WithFields::ThreadId => layer = layer.with_thread_ids(true),
            WithFields::ThreadName => layer = layer.with_thread_names(true),
            WithFields::InternalErrors => layer = layer.log_internal_errors(true),
        }
    }

    match format {
        Format::Compact => layer.compact().boxed(),
        Format::Pretty => layer.pretty().boxed(),
        Format::Json => layer.json().boxed(),
    }
}

fn init_env_filter(override_filter: Option<String>, level: &LogLevel) -> EnvFilter {
    EnvFilter::try_from_default_env()
        .or_else(|_| {
            // user wanted a specific filter, don't care about our internal whitelist
            // or, if no override give them the default whitelisted filter (most common)
            EnvFilter::try_new(override_filter.unwrap_or(format!("{level}")))
        })
        .expect("logger initialization failed")
}
