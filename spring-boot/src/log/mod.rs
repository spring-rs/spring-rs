mod config;

use crate::{app::AppBuilder, config::Configurable};
use anyhow::Context;
use config::TracingConfig;

pub(crate) struct LogPlugin;

impl Configurable for LogPlugin {
    fn config_prefix(&self) -> &str {
        "log"
    }
}

impl LogPlugin {
    pub(crate) fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<TracingConfig>(self)
            .context("tracing plugin config load failed")
            .unwrap();

        todo!()
    }
}
