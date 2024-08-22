use sea_streamer::SeaMessage;
use spring_boot::{app::App, async_trait};

#[async_trait]
pub trait FromMsg {
    async fn from_msg(msg: &SeaMessage, app: &App) -> Self;
}
