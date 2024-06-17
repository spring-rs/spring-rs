use tracing::log;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_log() {
    tracing_subscriber::registry().with(fmt::layer()).init();

    log::debug!("日志初始化完成，日志级别：");
}
