use crate::message::{ConnectionId, WebSocketMessage};
use dashmap::DashSet;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unique identifier for rooms
pub type RoomId = String;

/// Room management system
#[derive(Clone)]
pub struct RoomManager {
    /// All active rooms
    rooms: Arc<RwLock<DashSet<RoomId>>>,
}

impl RoomManager {
    /// Create a new room manager
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(RwLock::new(DashSet::new())),
        }
    }

    /// Create a new room
    pub async fn create_room(&self, room_id: RoomId) -> Result<(), RoomError> {
        let rooms = self.rooms.write().await;
        if rooms.contains(&room_id) {
            return Err(RoomError::RoomAlreadyExists(room_id.clone()));
        }

        // Need mutable access to insert
        drop(rooms);
        let rooms = self.rooms.write().await;
        rooms.insert(room_id.clone());
        tracing::info!("Created room: {}", room_id);
        Ok(())
    }

    /// Delete a room
    pub async fn delete_room(&self, room_id: &RoomId) -> Result<(), RoomError> {
        let rooms = self.rooms.write().await;
        if !rooms.remove(room_id).is_some() {
            return Err(RoomError::RoomNotFound(room_id.clone()));
        }
        tracing::info!("Deleted room: {}", room_id);
        Ok(())
    }

    /// Check if a room exists
    pub async fn room_exists(&self, room_id: &RoomId) -> bool {
        let rooms = self.rooms.read().await;
        rooms.contains(room_id)
    }

    /// Get all active rooms
    pub async fn list_rooms(&self) -> Vec<RoomId> {
        let rooms = self.rooms.read().await;
        rooms.iter().map(|r| r.clone()).collect()
    }

    /// Get room count
    pub async fn room_count(&self) -> usize {
        let rooms = self.rooms.read().await;
        rooms.len()
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual room state
#[derive(Clone)]
pub struct Room {
    /// Room ID
    pub id: RoomId,
    /// Connected clients in this room
    pub connections: Arc<RwLock<HashSet<ConnectionId>>>,
    /// Room metadata
    pub metadata: Arc<RwLock<std::collections::HashMap<String, String>>>,
}

impl Room {
    /// Create a new room
    pub fn new(room_id: RoomId) -> Self {
        Self {
            id: room_id,
            connections: Arc::new(RwLock::new(HashSet::new())),
            metadata: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Add a connection to the room
    pub async fn join(&self, connection_id: ConnectionId) -> Result<(), RoomError> {
        let mut connections = self.connections.write().await;
        if connections.contains(&connection_id) {
            return Err(RoomError::ConnectionAlreadyInRoom(connection_id));
        }
        connections.insert(connection_id);
        tracing::debug!("Connection {} joined room {}", connection_id, self.id);
        Ok(())
    }

    /// Remove a connection from the room
    pub async fn leave(&self, connection_id: &ConnectionId) -> Result<(), RoomError> {
        let mut connections = self.connections.write().await;
        if !connections.remove(connection_id) {
            return Err(RoomError::ConnectionNotInRoom(*connection_id));
        }
        tracing::debug!("Connection {} left room {}", connection_id, self.id);
        Ok(())
    }

    /// Get all connections in the room
    pub async fn get_connections(&self) -> HashSet<ConnectionId> {
        self.connections.read().await.clone()
    }

    /// Get connection count in the room
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Check if a connection is in the room
    pub async fn has_connection(&self, connection_id: &ConnectionId) -> bool {
        self.connections.read().await.contains(connection_id)
    }

    /// Broadcast a message to all connections in the room
    pub async fn broadcast(&self, message: WebSocketMessage) -> Result<usize, RoomError> {
        let connections = self.connections.read().await;
        let connection_count = connections.len();

        // Here we would send the message to all connections
        // This is a simplified version - in a real implementation,
        // you would need access to the actual WebSocket senders
        tracing::debug!(
            "Broadcasting message {} to {} connections in room {}",
            message.id,
            connection_count,
            self.id
        );

        Ok(connection_count)
    }

    /// Set room metadata
    pub async fn set_metadata<K: Into<String>, V: Into<String>>(
        &self,
        key: K,
        value: V,
    ) -> Result<(), RoomError> {
        let mut metadata = self.metadata.write().await;
        metadata.insert(key.into(), value.into());
        Ok(())
    }

    /// Get room metadata
    pub async fn get_metadata(&self, key: &str) -> Option<String> {
        let metadata = self.metadata.read().await;
        metadata.get(key).cloned()
    }

    /// Get all room metadata
    pub async fn get_all_metadata(&self) -> std::collections::HashMap<String, String> {
        self.metadata.read().await.clone()
    }
}

/// Room-related errors
#[derive(Debug, thiserror::Error)]
pub enum RoomError {
    #[error("Room {0} already exists")]
    RoomAlreadyExists(RoomId),

    #[error("Room {0} not found")]
    RoomNotFound(RoomId),

    #[error("Connection {0} already in room")]
    ConnectionAlreadyInRoom(ConnectionId),

    #[error("Connection {0} not in room")]
    ConnectionNotInRoom(ConnectionId),

    #[error("Room operation failed: {0}")]
    OperationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_room_creation() {
        let room = Room::new("test_room".to_string());
        assert_eq!(room.id, "test_room");
        assert_eq!(room.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_room_join_leave() {
        let room = Room::new("test_room".to_string());
        let connection_id = uuid::Uuid::new_v4();

        // Join room
        assert!(room.join(connection_id).await.is_ok());
        assert_eq!(room.connection_count().await, 1);
        assert!(room.has_connection(&connection_id).await);

        // Leave room
        assert!(room.leave(&connection_id).await.is_ok());
        assert_eq!(room.connection_count().await, 0);
        assert!(!room.has_connection(&connection_id).await);
    }

    #[tokio::test]
    async fn test_room_metadata() {
        let room = Room::new("test_room".to_string());

        // Set metadata
        assert!(room.set_metadata("key1", "value1").await.is_ok());
        assert_eq!(room.get_metadata("key1").await, Some("value1".to_string()));

        // Get all metadata
        let all_metadata = room.get_all_metadata().await;
        assert_eq!(all_metadata.get("key1"), Some(&"value1".to_string()));
    }

    #[tokio::test]
    async fn test_room_manager() {
        let manager = RoomManager::new();
        let room_id = "room1".to_string();

        // Create room
        assert!(manager.create_room(room_id.clone()).await.is_ok());
        assert!(manager.room_exists(&room_id).await);

        // Try to create duplicate room
        assert!(manager.create_room(room_id.clone()).await.is_err());

        // List rooms
        let rooms = manager.list_rooms().await;
        assert_eq!(rooms.len(), 1);
        assert_eq!(rooms[0], room_id);

        // Delete room
        assert!(manager.delete_room(&room_id).await.is_ok());
        assert!(!manager.room_exists(&room_id).await);
    }
}
