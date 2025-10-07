# Spring-RS WebSocket 插件

[![Crates.io](https://img.shields.io/crates/v/spring-websocket.svg)](https://crates.io/crates/spring-websocket)
[![Documentation](https://docs.rs/spring-websocket/badge.svg)](https://docs.rs/spring-websocket)

Spring-RS 框架的 WebSocket 插件为 Web 应用程序提供实时、双向通信功能。

## 特性

- 🚀 **高性能**: 基于 tokio-tungstenite 实现最佳性能
- 🔒 **类型安全**: 完整的类型安全与 serde 序列化
- 🏠 **房间管理**: 内置聊天室和群组消息支持
- 📊 **连接管理**: 自动连接生命周期和健康监控
- 🔧 **灵活配置**: 全面的 TOML 配置文件
- 🎯 **消息处理**: 可扩展的消息类型系统与自定义处理器
- 📈 **监控**: 内置统计和健康检查
- 🔒 **安全**: 可选认证和速率限制

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
spring-websocket = "0.4.0"
```

## 快速开始

### 1. 基本设置

```rust
use spring::{auto_config, App};
use spring_web::{get, WebConfigurator, WebPlugin};
use spring_websocket::WebSocketPlugin;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .add_plugin(WebSocketPlugin)
        .run()
        .await;

    println!("WebSocket 服务器启动成功");
}
```

### 2. 配置

创建 `config/app.toml` 文件：

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

### 3. 自定义消息处理器

```rust
use spring_websocket::handler::{MessageHandler, MessageHandlerBuilder};
use spring_websocket::message::{WebSocketMessage, MessageType, ConnectionId};
use spring_websocket::manager::WebSocketManager;
use std::error::Error as StdError;

struct ChatHandler;

impl MessageHandler for ChatHandler {
    fn handle(
        &self,
        message: WebSocketMessage,
        connection_id: ConnectionId,
        _manager: &WebSocketManager,
    ) -> Result<(), Box<dyn StdError>> {
        match message.message_type {
            MessageType::Text => {
                if let Some(text) = message.payload.as_text() {
                    println!("收到消息: {}", text);
                }
            }
            _ => {}
        }
        Ok(())
    }
}
```

## 架构

### 核心组件

- **WebSocketManager**: 中央连接和消息管理
- **ConnectionState**: 单个连接状态跟踪
- **MessageHandler**: 不同消息类型的处理trait
- **RoomManager**: 聊天室和群组管理
- **WebSocketConfig**: 配置管理

### 消息流程

1. **连接**: 客户端通过 WebSocket 升级连接
2. **消息接收**: 消息解析并路由到处理器
3. **处理器处理**: 基于消息类型的自定义逻辑执行
4. **响应**: 处理器可以向客户端发送响应
5. **清理**: 自动连接清理和资源管理

## 消息类型

WebSocket 插件支持多种内置消息类型：

| 类型 | 描述 | 载荷 |
|------|------|------|
| `Text` | 纯文本消息 | `String` |
| `Binary` | 二进制数据 | `Vec<u8>` |
| `RoomJoin` | 加入聊天室 | `String` (房间ID) |
| `RoomLeave` | 离开聊天室 | `String` (房间ID) |
| `RoomBroadcast` | 房间广播 | `String` (消息) |
| `Private` | 私信 | `String` (消息) |
| `Ping` | 健康检查 | - |
| `Pong` | 健康响应 | - |
| `System` | 系统消息 | `String` (消息) |

## 配置选项

| 选项 | 默认值 | 描述 |
|------|--------|------|
| `endpoint` | `"/ws"` | WebSocket 端点路径 |
| `max_connections` | `10000` | 最大并发连接数 |
| `max_message_size` | `65536` | 最大消息大小（字节） |
| `idle_timeout` | `300` | 连接空闲超时（秒） |
| `ping_interval` | `30` | Ping 间隔（秒） |
| `max_rooms` | `1000` | 最大房间数 |
| `compression` | `true` | 启用消息压缩 |
| `rate_limit` | - | 速率限制配置 |
| `auth` | - | 认证配置 |

## 高级用法

### 房间管理

```rust
use spring_websocket::room::{Room, RoomManager, RoomId};

// 创建和管理房间
let room_manager = RoomManager::new();
room_manager.create_room("general".to_string()).await?;

// 加入/离开房间
let room = Room::new("general".to_string());
room.join(connection_id).await?;
room.leave(&connection_id).await?;
```

### 自定义消息类型

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

### 广播消息

```rust
use spring_websocket::utils;

// 广播给所有连接
utils::broadcast_message(WebSocketMessage::new_system("服务器将在5分钟后重启")).await?;

// 发送给特定连接
utils::send_to_connection(connection_id, message).await?;
```

### 统计和监控

```rust
use spring_websocket::utils;

let manager = utils::get_manager().unwrap();
println!("活跃连接数: {}", manager.get_connection_count());
println!("连接ID列表: {:?}", manager.get_connection_ids());
```

## 示例

查看 `examples/websocket-example/` 目录中的完整聊天应用示例。

## 性能

WebSocket 插件针对以下方面进行了优化：

- **低延迟**: 最小化消息处理开销
- **高吞吐**: 高效连接和消息处理
- **内存安全**: Rust 类型系统保证零成本抽象
- **可扩展性**: 支持数千并发连接

## 安全性

### 认证

```toml
[websocket.auth]
token_header = "Authorization"
jwt_secret = "your-secret-key"
token_expiry = 3600
```

### 速率限制

```toml
[websocket.rate_limit]
messages_per_minute = 60
connections_per_ip = 10
ban_duration = 300
```

## 故障排除

### 常见问题

1. **连接失败**: 检查端点配置和防火墙
2. **消息未接收**: 验证消息格式和处理器注册
3. **性能问题**: 监控连接数并调整限制
4. **内存泄漏**: 检查连接清理逻辑

### 调试日志

在配置中启用调试日志：

```toml
[logging]
level = "debug"
```

## 贡献

欢迎社区专家贡献自己的插件！请查看主 [Spring-RS 仓库](https://github.com/spring-rs/spring-rs) 获取指南。

## 许可证

基于 MIT 许可证。详见 [LICENSE](LICENSE) 文件。
