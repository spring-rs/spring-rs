use tracing::log;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_log() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    log::debug!("log init");
}
