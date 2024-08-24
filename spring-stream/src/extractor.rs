use sea_streamer::Message;
use sea_streamer::MessageHeader;
use sea_streamer::SeaMessage;
use sea_streamer::SeqNo;
use sea_streamer::ShardId;
use sea_streamer::SharedMessage;
use sea_streamer::StreamKey;
use sea_streamer::Timestamp;
use spring_boot::app::App;

pub trait FromMsg {
    fn from_msg(msg: &SeaMessage, app: &App) -> Self;
}

impl FromMsg for StreamKey {
    fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.stream_key()
    }
}

impl FromMsg for SeqNo {
    fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.sequence()
    }
}

impl FromMsg for ShardId {
    fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.shard_id()
    }
}

impl FromMsg for Timestamp {
    fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.timestamp()
    }
}

impl FromMsg for MessageHeader {
    fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        MessageHeader::new(
            msg.stream_key(),
            msg.shard_id(),
            msg.sequence(),
            msg.timestamp(),
        )
    }
}

impl FromMsg for SharedMessage {
    fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        msg.to_owned()
    }
}

#[cfg(feature = "json")]
pub struct Json<T>(pub T);

#[cfg(feature = "json")]
impl<T> FromMsg for Json<T>
where
    T: serde::de::DeserializeOwned,
{
    fn from_msg(msg: &SeaMessage, _app: &App) -> Self {
        let value = msg
            .message()
            .deserialize_json()
            .expect("stream message parse as json failed");
        Json(value)
    }
}
