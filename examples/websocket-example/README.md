# Spring-RS WebSocket Example

This example demonstrates how to use the spring-websocket plugin to create real-time WebSocket applications with the Spring-RS framework.

## Features

- **Real-time Communication**: Full-duplex WebSocket communication
- **Room Management**: Support for chat rooms and group messaging
- **Connection Management**: Automatic connection lifecycle management
- **Message Handling**: Flexible message type system with custom handlers
- **Health Monitoring**: Built-in health checks and statistics
- **Configuration**: Comprehensive TOML-based configuration

## Project Structure

```
websocket-example/
├── src/main.rs          # Main application with WebSocket handlers
├── config/app.toml      # Configuration file
└── README.md           # This file
```

## Configuration

The WebSocket plugin is configured via the `config/app.toml` file:

```toml
[websocket]
endpoint = "/ws"                    # WebSocket endpoint path
max_connections = 1000             # Maximum concurrent connections
max_message_size = 65536           # Maximum message size in bytes
idle_timeout = 300                 # Connection idle timeout in seconds
ping_interval = 30                 # Ping interval in seconds
max_rooms = 100                    # Maximum number of rooms
compression = true                 # Enable message compression

# Optional: Rate limiting
[websocket.rate_limit]
messages_per_minute = 60
connections_per_ip = 10
ban_duration = 300

# Optional: Authentication
[websocket.auth]
token_header = "Authorization"
jwt_secret = "your-secret-key"
token_expiry = 3600
```

## Usage

### 1. Basic Setup

Add the WebSocket plugin to your application:

```rust
use spring_websocket::WebSocketPlugin;

let app = App::new()
    .add_plugin(WebPlugin)        // Required for HTTP server
    .add_plugin(WebSocketPlugin)  // WebSocket functionality
    .run()
    .await;
```

### 2. WebSocket Endpoints

Use the `websocket` macro to create WebSocket endpoints:

```rust
use spring_websocket::websocket;

#[websocket("/ws/chat")]
async fn chat_websocket_handler() -> impl spring_web::axum::response::IntoResponse {
    "WebSocket endpoint"
}
```

### 3. Message Handling

Create custom message handlers for different message types:

```rust
use spring_websocket::handler::{MessageHandler, MessageHandlerBuilder};
use spring_websocket::message::{WebSocketMessage, MessageType, ConnectionId};
use spring_websocket::manager::WebSocketManager;

struct ChatMessageHandler;

impl MessageHandler for ChatMessageHandler {
    async fn handle(
        &self,
        message: WebSocketMessage,
        connection_id: ConnectionId,
        manager: &WebSocketManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.message_type {
            MessageType::Text => {
                if let Some(text) = message.payload.as_text() {
                    println!("Received: {}", text);

                    // Echo the message back
                    let response = WebSocketMessage::new_text(&format!("Echo: {}", text));
                    manager.send_message_to_connection(connection_id, response).await?;
                }
            }
            MessageType::RoomJoin => {
                // Handle room joining logic
            }
            _ => {}
        }
        Ok(())
    }
}
```

### 4. Sending Messages

Use the utility functions to send messages programmatically:

```rust
use spring_websocket::utils;

// Send to specific connection
utils::send_to_connection(connection_id, message).await?;

// Broadcast to all connections
utils::broadcast_message(message).await?;

// Get manager instance
let manager = utils::get_manager();
```

## Message Types

The WebSocket plugin supports several built-in message types:

- **Text**: Plain text messages
- **Binary**: Binary data messages
- **RoomJoin**: Join a chat room
- **RoomLeave**: Leave a chat room
- **RoomBroadcast**: Broadcast to a room
- **Private**: Send private message to specific connection
- **Ping**: Ping message for connection health
- **Pong**: Response to ping
- **System**: System messages

## Built-in Handlers

The plugin includes default handlers for common operations:

- **RoomJoinHandler**: Handles room joining
- **RoomLeaveHandler**: Handles room leaving
- **RoomBroadcastHandler**: Handles room broadcasting
- **PrivateMessageHandler**: Handles private messaging
- **PingHandler**: Handles ping/pong for connection health
- **EchoHandler**: Echoes messages back (useful for testing)

## API Endpoints

The example includes these HTTP endpoints:

- `GET /` - WebSocket chat client interface
- `GET /api/ws/stats` - WebSocket connection statistics
- `GET /ws/health` - WebSocket health check

## Running the Example

1. **Build the project**:
   ```bash
   cargo build --example websocket-example
   ```

2. **Run the example**:
   ```bash
   cargo run --example websocket-example
   ```

3. **Open your browser** and navigate to `http://localhost:8080`

4. **Open browser developer tools** to see WebSocket messages in the console

## Testing WebSocket Connection

You can test the WebSocket connection using various tools:

### Using JavaScript (Browser Console)

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => console.log('Connected');
ws.onmessage = (event) => console.log('Received:', JSON.parse(event.data));
ws.onclose = () => console.log('Disconnected');

// Send a message
ws.send(JSON.stringify({
    message_type: 'Text',
    payload: { Text: 'Hello WebSocket!' },
    timestamp: Date.now()
}));
```

### Using curl (for HTTP endpoints)

```bash
# Get WebSocket statistics
curl http://localhost:8080/api/ws/stats

# Health check
curl http://localhost:8080/ws/health
```

### Using WebSocket Client Tools

You can also use dedicated WebSocket client tools like:
- **WebSocket King** (VS Code extension)
- **wscat** (npm package)
- **Postman** (has WebSocket support)

## Customization

### Custom Message Types

You can define your own message types:

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CustomMessageType {
    ChatMessage,
    UserJoined,
    UserLeft,
    GameMove,
    // Add your own types
}
```

### Custom Handlers

Create handlers for your custom message types:

```rust
struct GameHandler;

impl MessageHandler for GameHandler {
    async fn handle(
        &self,
        message: WebSocketMessage,
        connection_id: ConnectionId,
        manager: &WebSocketManager,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Handle game-specific messages
        Ok(())
    }
}
```

## Performance Considerations

- **Connection Limits**: Configure `max_connections` based on your server capacity
- **Message Size**: Set `max_message_size` to prevent memory issues
- **Ping/Pong**: Adjust `ping_interval` for connection health monitoring
- **Compression**: Enable `compression` to reduce bandwidth usage
- **Rate Limiting**: Use `rate_limit` to prevent abuse

## Troubleshooting

### Common Issues

1. **Connection Failed**: Check if the WebSocket endpoint is correctly configured
2. **Messages Not Received**: Verify message format matches expected structure
3. **Performance Issues**: Monitor connection count and adjust limits
4. **Memory Usage**: Check message sizes and implement cleanup logic

### Debug Logging

Enable debug logging to troubleshoot issues:

```toml
[logging]
level = "debug"
```

## Next Steps

- Explore room management features for multi-user applications
- Implement authentication for secure WebSocket connections
- Add rate limiting for production environments
- Integrate with other Spring-RS plugins (Redis, Database, etc.)
- Consider clustering for horizontal scaling
