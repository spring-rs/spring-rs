use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Unique identifier for WebSocket connections
pub type ConnectionId = Uuid;

/// Unique identifier for WebSocket messages
pub type MessageId = Uuid;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MessageType {
    /// Text message
    Text,
    /// Binary message
    Binary,
    /// Ping message
    Ping,
    /// Pong message
    Pong,
    /// Close message
    Close,
    /// Room join message
    RoomJoin,
    /// Room leave message
    RoomLeave,
    /// Room broadcast message
    RoomBroadcast,
    /// Private message
    Private,
    /// System message
    System,
}

/// WebSocket message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Unique message ID
    pub id: MessageId,
    /// Message type
    pub message_type: MessageType,
    /// Target connection ID (for private messages)
    pub target_connection_id: Option<ConnectionId>,
    /// Target room ID (for room messages)
    pub target_room_id: Option<String>,
    /// Message payload
    pub payload: MessagePayload,
    /// Message timestamp
    pub timestamp: u64,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl WebSocketMessage {
    /// Create a new text message
    pub fn new_text<S: Into<String>>(content: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: MessageType::Text,
            target_connection_id: None,
            target_room_id: None,
            payload: MessagePayload::Text(content.into()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new binary message
    pub fn new_binary(data: Vec<u8>) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: MessageType::Binary,
            target_connection_id: None,
            target_room_id: None,
            payload: MessagePayload::Binary(data),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Create a room broadcast message
    pub fn new_room_broadcast<S: Into<String>>(room_id: S, content: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: MessageType::RoomBroadcast,
            target_connection_id: None,
            target_room_id: Some(room_id.into()),
            payload: MessagePayload::Text(content.into()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Create a private message
    pub fn new_private<S: Into<String>>(target_id: ConnectionId, content: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: MessageType::Private,
            target_connection_id: Some(target_id),
            target_room_id: None,
            payload: MessagePayload::Text(content.into()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Create a system message
    pub fn new_system<S: Into<String>>(content: S) -> Self {
        Self {
            id: Uuid::new_v4(),
            message_type: MessageType::System,
            target_connection_id: None,
            target_room_id: None,
            payload: MessagePayload::Text(content.into()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata to the message
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Set target room
    pub fn with_room<S: Into<String>>(mut self, room_id: S) -> Self {
        self.target_room_id = Some(room_id.into());
        self
    }

    /// Set target connection
    pub fn with_target(mut self, connection_id: ConnectionId) -> Self {
        self.target_connection_id = Some(connection_id);
        self
    }
}

/// Message payload variants
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessagePayload {
    /// Text payload
    Text(String),
    /// Binary payload
    Binary(Vec<u8>),
    /// JSON payload
    Json(serde_json::Value),
    /// Structured data payload
    Structured(HashMap<String, serde_json::Value>),
}

impl MessagePayload {
    /// Get payload as text
    pub fn as_text(&self) -> Option<&str> {
        match self {
            MessagePayload::Text(text) => Some(text),
            _ => None,
        }
    }

    /// Get payload as binary
    pub fn as_binary(&self) -> Option<&Vec<u8>> {
        match self {
            MessagePayload::Binary(data) => Some(data),
            _ => None,
        }
    }

    /// Get payload as JSON
    pub fn as_json(&self) -> Option<&serde_json::Value> {
        match self {
            MessagePayload::Json(value) => Some(value),
            _ => None,
        }
    }

    /// Get payload as structured data
    pub fn as_structured(&self) -> Option<&HashMap<String, serde_json::Value>> {
        match self {
            MessagePayload::Structured(data) => Some(data),
            _ => None,
        }
    }

    /// Convert payload to JSON string
    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Convert payload from JSON string
    pub fn from_json_string(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

impl From<String> for MessagePayload {
    fn from(text: String) -> Self {
        MessagePayload::Text(text)
    }
}

impl From<Vec<u8>> for MessagePayload {
    fn from(data: Vec<u8>) -> Self {
        MessagePayload::Binary(data)
    }
}

impl From<serde_json::Value> for MessagePayload {
    fn from(value: serde_json::Value) -> Self {
        MessagePayload::Json(value)
    }
}

impl From<HashMap<String, serde_json::Value>> for MessagePayload {
    fn from(data: HashMap<String, serde_json::Value>) -> Self {
        MessagePayload::Structured(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = WebSocketMessage::new_text("Hello, WebSocket!");
        assert_eq!(msg.message_type, MessageType::Text);
        assert!(matches!(msg.payload, MessagePayload::Text(_)));
    }

    #[test]
    fn test_room_broadcast() {
        let msg = WebSocketMessage::new_room_broadcast("room1", "Hello room!");
        assert_eq!(msg.message_type, MessageType::RoomBroadcast);
        assert_eq!(msg.target_room_id, Some("room1".to_string()));
    }

    #[test]
    fn test_private_message() {
        let target_id = Uuid::new_v4();
        let msg = WebSocketMessage::new_private(target_id, "Private message");
        assert_eq!(msg.message_type, MessageType::Private);
        assert_eq!(msg.target_connection_id, Some(target_id));
    }

    #[test]
    fn test_metadata() {
        let msg = WebSocketMessage::new_text("test")
            .with_metadata("author", "test_user")
            .with_metadata("priority", "high");

        assert_eq!(msg.metadata.get("author"), Some(&"test_user".to_string()));
        assert_eq!(msg.metadata.get("priority"), Some(&"high".to_string()));
    }

    #[test]
    fn test_payload_conversions() {
        let text_payload = MessagePayload::from("Hello".to_string());
        assert_eq!(text_payload.as_text(), Some("Hello"));

        let binary_payload = MessagePayload::from(vec![1, 2, 3]);
        assert_eq!(binary_payload.as_binary(), Some(&vec![1, 2, 3]));
    }
}
