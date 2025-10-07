use spring::{auto_config, App};
use spring_web::{get, WebConfigurator, WebPlugin};
use spring_websocket::WebSocketPlugin;
use spring_websocket::message::{WebSocketMessage, MessageType};
use spring_websocket::MessageHandler;
use spring_websocket::utils;
// use serde::{Deserialize, Serialize}; // Not currently used
use uuid::Uuid;
use std::collections::HashMap;
use std::error::Error as StdError;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    // Create custom message handler for chat messages
    let _chat_handler = ChatMessageHandler::new();

    App::new()
        .add_plugin(WebPlugin)
        .add_plugin(WebSocketPlugin)
        .run()
        .await;

    println!("WebSocket example completed successfully");
}

// WebSocket endpoint is handled by the WebSocketPlugin configuration
// The endpoint is configured in config/app.toml as "/ws"

/// Custom handler for chat messages
struct ChatMessageHandler {
    // Note: chat_rooms field is for demonstration purposes
    // In a real implementation, this would track active chat rooms
    #[allow(dead_code)]
    chat_rooms: HashMap<String, Vec<Uuid>>,
}

impl ChatMessageHandler {
    fn new() -> Self {
        Self {
            chat_rooms: HashMap::new(),
        }
    }
}

impl MessageHandler for ChatMessageHandler {
    fn handle(
        &self,
        message: WebSocketMessage,
        connection_id: spring_websocket::message::ConnectionId,
        _manager: &spring_websocket::manager::WebSocketManager,
    ) -> Result<(), Box<dyn StdError>> {
        match message.message_type {
            MessageType::RoomJoin => {
                if let Some(room_id) = message.payload.as_text() {
                    println!("Connection {} joining chat room: {}", connection_id, room_id);
                }
            }
            MessageType::Text => {
                if let Some(text) = message.payload.as_text() {
                    println!("Chat message from {}: {}", connection_id, text);
                }
            }
            _ => {
                // Handle other message types with default behavior
            }
        }

        Ok(())
    }
}

#[get("/")]
async fn index() -> impl spring_web::axum::response::IntoResponse {
    r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Spring-RS WebSocket Example</title>
    </head>
    <body>
        <h1>Spring-RS WebSocket Chat Example</h1>
        <div>
            <input type="text" id="messageInput" placeholder="Type a message...">
            <button onclick="sendMessage()">Send</button>
        </div>
        <div id="messages"></div>

        <script>
            const ws = new WebSocket('ws://localhost:8080/ws/chat');
            const messagesDiv = document.getElementById('messages');

            ws.onopen = function(event) {
                messagesDiv.innerHTML += '<p>Connected to WebSocket server</p>';
            };

            ws.onmessage = function(event) {
                const message = JSON.parse(event.data);
                messagesDiv.innerHTML += '<p><strong>' + message.message_type + ':</strong> ' +
                    (message.payload.Text || message.payload) + '</p>';
            };

            ws.onclose = function(event) {
                messagesDiv.innerHTML += '<p>Disconnected from WebSocket server</p>';
            };

            function sendMessage() {
                const input = document.getElementById('messageInput');
                const message = {
                    id: crypto.randomUUID(),
                    message_type: 'Text',
                    payload: { Text: input.value },
                    timestamp: Date.now(),
                    metadata: {}
                };

                ws.send(JSON.stringify(message));
                input.value = '';
            }

            // Allow sending with Enter key
            document.getElementById('messageInput').addEventListener('keypress', function(e) {
                if (e.key === 'Enter') {
                    sendMessage();
                }
            });
        </script>
    </body>
    </html>
    "#
}

#[get("/api/ws/stats")]
async fn websocket_stats() -> impl spring_web::axum::response::IntoResponse {
    use spring_web::axum::Json;

    if let Some(manager) = utils::get_manager() {
        let stats = serde_json::json!({
            "connection_count": manager.get_connection_count(),
            "connection_ids": manager.get_connection_ids(),
            "active": true
        });
        Json(stats)
    } else {
        Json(serde_json::json!({
            "error": "WebSocket manager not available",
            "active": false
        }))
    }
}
