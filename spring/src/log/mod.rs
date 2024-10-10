mod config;

use crate::app::AppBuilder;
use crate::config::ConfigRegistry;
use config::{Format, LoggerConfig, TimeStyle, WithFields};
use std::ops::Deref;
use std::sync::OnceLock;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::fmt::time::{ChronoLocal, ChronoUtc, FormatTime, SystemTime, Uptime};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::reload::{self, Handle};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;
use tracing_subscriber::{
    fmt::{self, MakeWriter},
    Layer,
};

type BoxLayer<S> = Box<dyn Layer<S> + Send + Sync + 'static>;

pub(crate) struct LogPlugin;

impl LogPlugin {
    pub(crate) fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<LoggerConfig>()
            .expect("tracing plugin config load failed");

        if config.pretty_backtrace {
            std::env::set_var("RUST_BACKTRACE", "1");
            log::warn!(
                "pretty backtraces are enabled (this is great for development but has a runtime cost for production. disable with `logger.pretty_backtrace` in your config)"
            );
        }

        let layers = config.config_subscriber();

        let env_filter = config.build_env_filter();

        let (layers, layers_reload_handler) = reload::Layer::new(layers);

        tracing_subscriber::registry()
            .with(layers)
            .with(env_filter)
            .init();

        app.add_component(LayersReloader(layers_reload_handler));
    }
}

// Keep nonblocking file appender work guard
static NONBLOCKING_WORK_GUARD_KEEP: OnceLock<WorkerGuard> = OnceLock::new();

impl LoggerConfig {
    fn config_subscriber(&self) -> Vec<BoxLayer<Registry>> {
        let mut layers = Vec::new();
        if let Some(file_logger) = &self.file_appender {
            if file_logger.enable {
                let file_appender = tracing_appender::rolling::Builder::default()
                    .max_log_files(file_logger.max_log_files)
                    .filename_prefix(&file_logger.filename_prefix)
                    .filename_suffix(&file_logger.filename_suffix)
                    .rotation(file_logger.rotation.clone().into())
                    .build(&file_logger.dir)
                    .expect("logger file appender initialization failed");

                let file_appender_layer = if file_logger.non_blocking {
                    let (non_blocking_file_appender, work_guard) =
                        tracing_appender::non_blocking(file_appender);
                    NONBLOCKING_WORK_GUARD_KEEP.set(work_guard).unwrap();
                    self.build_fmt_layer(non_blocking_file_appender, &file_logger.format, false)
                } else {
                    self.build_fmt_layer(file_appender, &file_logger.format, false)
                };
                layers.push(file_appender_layer);
            }
        }

        if self.enable {
            layers.push(self.build_fmt_layer(std::io::stdout, &self.format, true));
        }

        layers
    }

    fn build_fmt_layer<W2>(
        &self,
        make_writer: W2,
        format: &Format,
        ansi: bool,
    ) -> BoxLayer<Registry>
    where
        W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
    {
        let LoggerConfig {
            time_style,
            time_pattern,
            ..
        } = &self;
        match time_style {
            TimeStyle::SystemTime => {
                self.build_layer_with_timer(make_writer, format, SystemTime, ansi)
            }
            TimeStyle::Uptime => {
                self.build_layer_with_timer(make_writer, format, Uptime::default(), ansi)
            }
            TimeStyle::ChronoLocal => self.build_layer_with_timer(
                make_writer,
                format,
                ChronoLocal::new(time_pattern.to_string()),
                ansi,
            ),
            TimeStyle::ChronoUtc => self.build_layer_with_timer(
                make_writer,
                format,
                ChronoUtc::new(time_pattern.to_string()),
                ansi,
            ),
            TimeStyle::None => self.build_layer_without_timer(make_writer, format, ansi),
        }
    }

    fn build_layer_with_timer<W2, T>(
        &self,
        make_writer: W2,
        format: &Format,
        timer: T,
        ansi: bool,
    ) -> BoxLayer<Registry>
    where
        W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
        T: FormatTime + Sync + Send + 'static,
    {
        let mut layer = fmt::Layer::default()
            .with_ansi(ansi)
            .with_writer(make_writer)
            .with_timer(timer);

        for field in &self.with_fields {
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

    fn build_layer_without_timer<W2>(
        &self,
        make_writer: W2,
        format: &Format,
        ansi: bool,
    ) -> BoxLayer<Registry>
    where
        W2: for<'writer> MakeWriter<'writer> + Sync + Send + 'static,
    {
        let mut layer = fmt::Layer::default()
            .with_ansi(ansi)
            .with_writer(make_writer)
            .without_time();

        for field in &self.with_fields {
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

    fn build_env_filter(&self) -> EnvFilter {
        match EnvFilter::try_from_default_env() {
            Ok(env_filter) => env_filter,
            Err(_) => {
                let LoggerConfig {
                    override_filter,
                    level,
                    ..
                } = self;
                let directive = match override_filter {
                    Some(dir) => dir.into(),
                    None => format!("{level}"),
                };
                EnvFilter::try_new(directive).expect("logger initialization failed")
            }
        }
    }
}

#[derive(Clone)]
pub struct LayersReloader(Handle<Vec<BoxLayer<Registry>>, Registry>);

impl Deref for LayersReloader {
    type Target = Handle<Vec<BoxLayer<Registry>>, Registry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
