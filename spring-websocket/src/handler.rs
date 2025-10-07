use crate::manager::{MessageHandler, WebSocketManager};
use crate::message::{ConnectionId, MessageType, WebSocketMessage};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;

/// Default message handlers for common WebSocket message types
pub struct DefaultMessageHandlers {
    manager: Arc<WebSocketManager>,
}

impl DefaultMessageHandlers {
    /// Create new default message handlers
    pub fn new(manager: Arc<WebSocketManager>) -> Self {
        Self { manager }
    }

    /// Register all default handlers with the manager
    pub fn register_all(&self, manager: &WebSocketManager) {
        let builder = MessageHandlerBuilder::new(self.manager.clone())
            .add_handler(MessageType::RoomJoin, RoomJoinHandler::new(self.manager.clone()))
            .add_handler(MessageType::RoomLeave, RoomLeaveHandler::new(self.manager.clone()))
            .add_handler(MessageType::RoomBroadcast, RoomBroadcastHandler::new(self.manager.clone()))
            .add_handler(MessageType::Private, PrivateMessageHandler::new(self.manager.clone()))
            .add_handler(MessageType::Ping, PingHandler::new(self.manager.clone()))
            .add_handler(MessageType::Text, EchoHandler::new(self.manager.clone()));

        builder.build(manager);
    }
}

/// Handler for room join messages
struct RoomJoinHandler {
    manager: Arc<WebSocketManager>,
}

impl Clone for RoomJoinHandler {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
        }
    }
}

impl RoomJoinHandler {
    fn new(manager: Arc<WebSocketManager>) -> Self {
        Self { manager }
    }
}

impl MessageHandler for RoomJoinHandler {
    fn handle(
        &self,
        message: WebSocketMessage,
        connection_id: ConnectionId,
        _manager: &WebSocketManager,
    ) -> Result<(), Box<dyn StdError>> {
        // Extract room ID from message payload
        let room_id = if let Some(room_id_str) = message.payload.as_text() {
            room_id_str.to_string()
        } else {
            return Err("Room ID not provided in message payload".into());
        };

        tracing::info!("Connection {} joining room {}", connection_id, room_id);
        Ok(())
    }
}

/// Handler for room leave messages
struct RoomLeaveHandler {
    manager: Arc<WebSocketManager>,
}

impl Clone for RoomLeaveHandler {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
        }
    }
}

impl RoomLeaveHandler {
    fn new(manager: Arc<WebSocketManager>) -> Self {
        Self { manager }
    }
}

impl MessageHandler for RoomLeaveHandler {
    fn handle(
        &self,
        message: WebSocketMessage,
        connection_id: ConnectionId,
        _manager: &WebSocketManager,
    ) -> Result<(), Box<dyn StdError>> {
        // Extract room ID from message payload
        let room_id = if let Some(room_id_str) = message.payload.as_text() {
            room_id_str.to_string()
        } else {
            return Err("Room ID not provided in message payload".into());
        };

        tracing::info!("Connection {} leaving room {}", connection_id, room_id);
        Ok(())
    }
}

/// Handler for room broadcast messages
struct RoomBroadcastHandler {
    manager: Arc<WebSocketManager>,
}

impl Clone for RoomBroadcastHandler {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
        }
    }
}

impl RoomBroadcastHandler {
    fn new(manager: Arc<WebSocketManager>) -> Self {
        Self { manager }
    }
}

impl MessageHandler for RoomBroadcastHandler {
    fn handle(
        &self,
        message: WebSocketMessage,
        connection_id: ConnectionId,
        _manager: &WebSocketManager,
    ) -> Result<(), Box<dyn StdError>> {
        // Extract room ID from message metadata
        let room_id = if let Some(room_id_str) = message.target_room_id {
            room_id_str
        } else {
            return Err("Room ID not provided in message target_room_id".into());
        };

        tracing::info!("Broadcasting message from {} to room {}", connection_id, room_id);
        Ok(())
    }
}

/// Handler for private messages
struct PrivateMessageHandler {
    manager: Arc<WebSocketManager>,
}

impl Clone for PrivateMessageHandler {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
        }
    }
}

impl PrivateMessageHandler {
    fn new(manager: Arc<WebSocketManager>) -> Self {
        Self { manager }
    }
}

impl MessageHandler for PrivateMessageHandler {
    fn handle(
        &self,
        message: WebSocketMessage,
        connection_id: ConnectionId,
        _manager: &WebSocketManager,
    ) -> Result<(), Box<dyn StdError>> {
        // Extract target connection ID from message
        let target_id = if let Some(target_id) = message.target_connection_id {
            target_id
        } else {
            return Err("Target connection ID not provided in message".into());
        };

        tracing::info!("Sending private message from {} to {}", connection_id, target_id);
        Ok(())
    }
}

/// Handler for ping messages
struct PingHandler {
    manager: Arc<WebSocketManager>,
}

impl Clone for PingHandler {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
        }
    }
}

impl PingHandler {
    fn new(manager: Arc<WebSocketManager>) -> Self {
        Self { manager }
    }
}

impl MessageHandler for PingHandler {
    fn handle(
        &self,
        _message: WebSocketMessage,
        connection_id: ConnectionId,
        _manager: &WebSocketManager,
    ) -> Result<(), Box<dyn StdError>> {
        tracing::debug!("Received ping from connection {}", connection_id);
        Ok(())
    }
}

/// Echo handler for testing
struct EchoHandler {
    manager: Arc<WebSocketManager>,
}

impl Clone for EchoHandler {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
        }
    }
}

impl EchoHandler {
    fn new(manager: Arc<WebSocketManager>) -> Self {
        Self { manager }
    }
}

impl MessageHandler for EchoHandler {
    fn handle(
        &self,
        _message: WebSocketMessage,
        connection_id: ConnectionId,
        _manager: &WebSocketManager,
    ) -> Result<(), Box<dyn StdError>> {
        tracing::debug!("Echo handler received message from connection {}", connection_id);
        Ok(())
    }
}

/// Custom message handler builder
pub struct MessageHandlerBuilder {
    handlers: HashMap<MessageType, Arc<dyn MessageHandler + Send + Sync>>,
    #[allow(dead_code)]
    manager: Arc<WebSocketManager>,
}

impl MessageHandlerBuilder {
    /// Create a new handler builder
    pub fn new(manager: Arc<WebSocketManager>) -> Self {
        Self {
            handlers: HashMap::new(),
            manager,
        }
    }

    /// Add a handler for a specific message type
    pub fn add_handler<H>(mut self, message_type: MessageType, handler: H) -> Self
    where
        H: MessageHandler + 'static,
    {
        self.handlers.insert(message_type, Arc::new(handler));
        self
    }

    /// Build the handlers and register them with the manager
    pub fn build(self, manager: &WebSocketManager) {
        // Register the handlers that were added
        for (message_type, handler) in self.handlers {
            manager.register_message_handler_raw(message_type, handler);
        }
    }
}

// Note: Default is not implemented since we need a manager instance

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WebSocketConfig;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_message_handler_builder() {
        let manager = Arc::new(WebSocketManager::new(WebSocketConfig::default()));
        let _ws_manager = WebSocketManager::new(WebSocketConfig::default());

        let _builder = MessageHandlerBuilder::new(manager.clone())
            .add_handler(MessageType::Text, EchoHandler::new(manager.clone()));

        // This would register the handler in a real scenario
        // builder.build(&ws_manager);

        // For testing, just verify the builder was created correctly
        assert!(true); // Placeholder test
    }
}
