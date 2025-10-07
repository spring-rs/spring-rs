# Spring-RS WebSocket Plugin

[![Crates.io](https://img.shields.io/crates/v/spring-websocket.svg)](https://crates.io/crates/spring-websocket)
[![Documentation](https://docs.rs/spring-websocket/badge.svg)](https://docs.rs/spring-websocket)

The WebSocket plugin for [Spring-RS](https://github.com/spring-rs/spring-rs) framework provides real-time, bidirectional communication capabilities for web applications.

## Features

- 🚀 **High Performance**: Built on top of tokio-tungstenite for optimal performance
- 🔒 **Type Safe**: Full type safety with serde serialization
- 🏠 **Room Management**: Built-in support for chat rooms and group messaging
- 📊 **Connection Management**: Automatic connection lifecycle and health monitoring
- 🔧 **Flexible Configuration**: Comprehensive TOML-based configuration
- 🎯 **Message Handling**: Extensible message type system with custom handlers
- 📈 **Monitoring**: Built-in statistics and health checks
- 🔒 **Secure**: Optional authentication and rate limiting

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
spring-websocket = "0.1.0"
```

## Quick Start

### 1. Basic Setup

```rust
use spring::{auto_config, App};
use spring_web::{get, WebConfigurator, WebPlugin};
use spring_websocket::{websocket, WebSocketPlugin};

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    let app = App::new()
        .add_plugin(WebPlugin)
        .add_plugin(WebSocketPlugin)
        .run()
        .await;

    match app {
        Ok(_) => println!("WebSocket server started successfully"),
        Err(e) => eprintln!("Failed to start server: {:?}", e),
    }
}

#[websocket("/ws")]
async fn websocket_handler() -> impl spring_web::axum::response::IntoResponse {
    "WebSocket endpoint"
}
```

### 2. Configuration

Create a `config/app.toml` file:

```toml
[web]
binding = "0.0.0.0"
port = 8080

[websocket]
endpoint = "/ws"
max_connections = 1000
max_message_size = 65536
idle_timeout = 300
ping_interval = 30
compression = true
```

### 3. Custom Message Handler

```rust
use spring_websocket::handler::{MessageHandler, MessageHandlerBuilder};
use spring_websocket::message::{WebSocketMessage, MessageType, ConnectionId};
use spring_websocket::manager::WebSocketManager;
use std::error::Error as StdError;

struct ChatHandler;

impl MessageHandler for ChatHandler {
    async fn handle(
        &self,
        message: WebSocketMessage,
        connection_id: ConnectionId,
        manager: &WebSocketManager,
    ) -> Result<(), Box<dyn StdError>> {
        match message.message_type {
            MessageType::Text => {
                if let Some(text) = message.payload.as_text() {
                    println!("Received: {}", text);

                    // Echo back
                    let response = WebSocketMessage::new_text(&format!("Echo: {}", text));
                    manager.send_message_to_connection(connection_id, response).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Architecture

### Core Components

- **WebSocketManager**: Central connection and message management
- **ConnectionState**: Individual connection state tracking
- **MessageHandler**: Trait for handling different message types
- **RoomManager**: Chat room and group management
- **WebSocketConfig**: Configuration management

### Message Flow

1. **Connection**: Client connects via WebSocket upgrade
2. **Message Reception**: Messages parsed and routed to handlers
3. **Handler Processing**: Custom logic executed based on message type
4. **Response**: Handlers can send responses back to clients
5. **Cleanup**: Automatic connection cleanup and resource management

## Message Types

The plugin supports several built-in message types:

| Type | Description | Payload |
|------|-------------|---------|
| `Text` | Plain text messages | `String` |
| `Binary` | Binary data | `Vec<u8>` |
| `RoomJoin` | Join a chat room | `String` (room ID) |
| `RoomLeave` | Leave a chat room | `String` (room ID) |
| `RoomBroadcast` | Broadcast to room | `String` (message) |
| `Private` | Private message | `String` (message) |
| `Ping` | Health check | - |
| `Pong` | Health response | - |
| `System` | System messages | `String` (message) |

## Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `endpoint` | `"/ws"` | WebSocket endpoint path |
| `max_connections` | `10000` | Maximum concurrent connections |
| `max_message_size` | `65536` | Maximum message size in bytes |
| `idle_timeout` | `300` | Connection idle timeout in seconds |
| `ping_interval` | `30` | Ping interval in seconds |
| `max_rooms` | `1000` | Maximum number of rooms |
| `compression` | `true` | Enable message compression |
| `rate_limit` | - | Rate limiting configuration |
| `auth` | - | Authentication configuration |

## Advanced Usage

### Room Management

```rust
use spring_websocket::room::{Room, RoomManager, RoomId};

// Create and manage rooms
let room_manager = RoomManager::new();
room_manager.create_room("general".to_string()).await?;

// Join/leave rooms
let room = Room::new("general".to_string());
room.join(connection_id).await?;
room.leave(&connection_id).await?;
```

### Custom Message Types

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CustomMessageType {
    GameMove,
    PlayerJoined,
    PlayerLeft,
}

impl MessageType {
    pub const GAME_MOVE: MessageType = MessageType::Custom(1);
    pub const PLAYER_JOINED: MessageType = MessageType::Custom(2);
    pub const PLAYER_LEFT: MessageType = MessageType::Custom(3);
}
```

### Broadcasting

```rust
use spring_websocket::utils;

// Broadcast to all connections
utils::broadcast_message(WebSocketMessage::new_system("Server restart in 5 minutes")).await?;

// Send to specific connection
utils::send_to_connection(connection_id, message).await?;
```

### Statistics and Monitoring

```rust
use spring_websocket::utils;

let manager = utils::get_manager().unwrap();
println!("Active connections: {}", manager.get_connection_count());
println!("Connection IDs: {:?}", manager.get_connection_ids());
```

## Examples

See the `examples/websocket-example/` directory for a complete chat application example.

## Performance

The WebSocket plugin is optimized for:

- **Low Latency**: Minimal overhead message processing
- **High Throughput**: Efficient connection and message handling
- **Memory Safety**: Zero-cost abstractions with Rust's type system
- **Scalability**: Support for thousands of concurrent connections

## Security

### Authentication

```toml
[websocket.auth]
token_header = "Authorization"
jwt_secret = "your-secret-key"
token_expiry = 3600
```

### Rate Limiting

```toml
[websocket.rate_limit]
messages_per_minute = 60
connections_per_ip = 10
ban_duration = 300
```

## Troubleshooting

### Common Issues

1. **Connection Failed**: Check endpoint configuration and firewall
2. **Messages Not Received**: Verify message format and handler registration
3. **Performance Issues**: Monitor connection count and adjust limits
4. **Memory Leaks**: Check for proper connection cleanup

### Debug Logging

Enable debug logging in your configuration:

```toml
[logging]
level = "debug"
```

## Contributing

Contributions are welcome! Please see the main [Spring-RS repository](https://github.com/spring-rs/spring-rs) for guidelines.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) file for details.
