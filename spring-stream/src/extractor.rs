use sea_streamer::Buffer;
use sea_streamer::Message;
use sea_streamer::MessageHeader;
use sea_streamer::SeaMessage;
use sea_streamer::SeqNo;
use sea_streamer::ShardId;
use sea_streamer::SharedMessage;
use sea_streamer::StreamKey;
use sea_streamer::Timestamp;
use spring_boot::{app::App, async_trait};

#[async_trait]
pub trait FromMsg {
    async fn from_msg(msg: &SeaMessage, app: &App) -> Self;
}

#[async_trait]
impl FromMsg for StreamKey {
    async fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.stream_key()
    }
}

#[async_trait]
impl FromMsg for SeqNo {
    async fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.sequence()
    }
}

#[async_trait]
impl FromMsg for ShardId {
    async fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.shard_id()
    }
}

#[async_trait]
impl FromMsg for Timestamp {
    async fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.timestamp()
    }
}

#[async_trait]
impl FromMsg for MessageHeader {
    async fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        MessageHeader::new(
            msg.stream_key(),
            msg.shard_id(),
            msg.sequence(),
            msg.timestamp(),
        )
    }
}

#[async_trait]
impl FromMsg for SharedMessage {
    async fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.to_owned()
    }
}

#[cfg(feature = "json")]
pub struct Json<T>(pub T);

#[cfg(feature = "json")]
#[async_trait]
impl<T> FromMsg for Json<T>
where
    T: serde::de::DeserializeOwned,
{
    async fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        let value = msg
            .message()
            .deserialize_json()
            .expect("stream message parse as json failed");
        Json(value)
    }
}
