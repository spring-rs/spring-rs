//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io/docs/plugins/spring-websocket)
//! WebSocket plugin for spring-rs framework
//!
//! This plugin provides WebSocket functionality for spring-rs applications,
//! built on top of tokio-tungstenite and integrated with axum web framework.

pub mod config;
pub mod handler;
pub mod manager;
pub mod message;
pub mod room;

// Re-export key types and traits
pub use handler::MessageHandlerBuilder;
pub use manager::{MessageHandler, WebSocketManager};
pub use message::{WebSocketMessage, MessageType, ConnectionId, MessageId};

// pub use spring_macros::websocket; // TODO: Implement websocket macro

use crate::config::WebSocketConfig;
use crate::handler::DefaultMessageHandlers;
use axum::extract::{ConnectInfo, WebSocketUpgrade};
use axum::extract::ws::WebSocket;
use axum::response::Response;
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::plugin::{ComponentRegistry, MutableComponentRegistry};
use spring_web::WebConfigurator;
use spring::{
    app::AppBuilder,
    plugin::Plugin,
};
use std::net::SocketAddr;
use std::sync::Arc;

/// WebSocket Plugin
pub struct WebSocketPlugin;

#[async_trait]
impl Plugin for WebSocketPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<WebSocketConfig>()
            .expect("websocket plugin config load failed");

        let manager = WebSocketManager::new(config.clone());
        let manager_arc = Arc::new(manager);

        // Register the manager as a component
        app.add_component(manager_arc.clone());

        // Register default message handlers
        let handlers = DefaultMessageHandlers::new(manager_arc.clone());
        handlers.register_all(&manager_arc);

        // Register WebSocket upgrade handler with the web router
        app.add_router(config.create_router());
    }
}

impl WebSocketConfig {
    /// Create router for WebSocket endpoints
    pub fn create_router(&self) -> spring_web::Router {
        use axum::routing::get;
        use spring_web::Router;

        Router::new()
            .route(&self.endpoint, get(websocket_handler))
            .route("/ws/health", get(health_check))
    }
}

/// WebSocket upgrade handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    tracing::info!("WebSocket connection attempt from: {}", addr);
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

/// Handle individual WebSocket connection
async fn handle_socket(socket: WebSocket, addr: SocketAddr) {
    let manager = spring::App::global()
        .get_component::<Arc<WebSocketManager>>()
        .expect("WebSocketManager not found");

    if let Err(e) = manager.handle_connection(socket, addr).await {
        tracing::error!("WebSocket connection error from {}: {:?}", addr, e);
    }
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "WebSocket server is healthy"
}

/// WebSocket utilities and helpers
pub mod utils {
    use super::*;

    /// Send a message to a specific connection
    pub async fn send_to_connection(
        connection_id: crate::message::ConnectionId,
        message: crate::message::WebSocketMessage,
    ) -> spring::error::Result<()> {
        let manager = spring::App::global()
            .get_component::<Arc<WebSocketManager>>()
            .expect("WebSocketManager not found");

        manager.send_message_to_connection(connection_id, message).await.map_err(|e| {
            spring::error::AppError::OtherError(anyhow::anyhow!("WebSocket send error: {}", e))
        })
    }

    /// Broadcast a message to all connections
    pub async fn broadcast_message(
        message: crate::message::WebSocketMessage,
    ) -> spring::error::Result<usize> {
        let manager = spring::App::global()
            .get_component::<Arc<WebSocketManager>>()
            .expect("WebSocketManager not found");

        manager.broadcast_message(message).await.map_err(|e| {
            spring::error::AppError::OtherError(anyhow::anyhow!("WebSocket broadcast error: {}", e))
        })
    }

    /// Get WebSocket manager instance
    pub fn get_manager() -> Option<Arc<WebSocketManager>> {
        spring::App::global()
            .get_component::<Arc<WebSocketManager>>()
    }
}
