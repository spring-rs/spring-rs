use schemars::JsonSchema;
use serde::Deserialize;
use spring::config::Configurable;
use std::time::Duration;

spring::submit_config_schema!("websocket", WebSocketConfig);

/// WebSocket Plugin Configuration
#[derive(Debug, Configurable, JsonSchema, Deserialize, Clone)]
#[config_prefix = "websocket"]
pub struct WebSocketConfig {
    /// WebSocket endpoint path
    #[serde(default = "default_endpoint")]
    pub endpoint: String,

    /// Maximum number of concurrent connections
    #[serde(default = "default_max_connections")]
    pub max_connections: usize,

    /// Maximum message size in bytes
    #[serde(default = "default_max_message_size")]
    pub max_message_size: usize,

    /// Connection idle timeout in seconds
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    /// Ping interval in seconds
    #[serde(default = "default_ping_interval")]
    pub ping_interval: u64,

    /// Maximum number of rooms
    #[serde(default = "default_max_rooms")]
    pub max_rooms: usize,

    /// Enable message compression
    #[serde(default = "default_compression")]
    pub compression: bool,

    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,

    /// Authentication configuration
    pub auth: Option<AuthConfig>,
}

fn default_endpoint() -> String {
    "/ws".to_string()
}

fn default_max_connections() -> usize {
    10000
}

fn default_max_message_size() -> usize {
    65536 // 64KB
}

fn default_idle_timeout() -> u64 {
    300 // 5 minutes
}

fn default_ping_interval() -> u64 {
    30 // 30 seconds
}

fn default_max_rooms() -> usize {
    1000
}

fn default_compression() -> bool {
    true
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            endpoint: default_endpoint(),
            max_connections: default_max_connections(),
            max_message_size: default_max_message_size(),
            idle_timeout: default_idle_timeout(),
            ping_interval: default_ping_interval(),
            max_rooms: default_max_rooms(),
            compression: default_compression(),
            rate_limit: None,
            auth: None,
        }
    }
}

impl WebSocketConfig {
    /// Get idle timeout as Duration
    pub fn idle_timeout_duration(&self) -> Duration {
        Duration::from_secs(self.idle_timeout)
    }

    /// Get ping interval as Duration
    pub fn ping_interval_duration(&self) -> Duration {
        Duration::from_secs(self.ping_interval)
    }

    /// Check if rate limiting is enabled
    pub fn is_rate_limited(&self) -> bool {
        self.rate_limit.is_some()
    }

    /// Check if authentication is required
    pub fn requires_auth(&self) -> bool {
        self.auth.is_some()
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum messages per minute per connection
    pub messages_per_minute: u32,

    /// Maximum concurrent connections per IP
    pub connections_per_ip: u32,

    /// Ban duration in seconds after rate limit exceeded
    pub ban_duration: u64,
}

/// Authentication configuration
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct AuthConfig {
    /// Authentication token header name
    pub token_header: String,

    /// JWT secret for token verification
    pub jwt_secret: String,

    /// Token expiration time in seconds
    pub token_expiry: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WebSocketConfig::default();
        assert_eq!(config.endpoint, "/ws");
        assert_eq!(config.max_connections, 10000);
        assert_eq!(config.max_message_size, 65536);
        assert_eq!(config.idle_timeout, 300);
        assert_eq!(config.ping_interval, 30);
        assert_eq!(config.max_rooms, 1000);
        assert!(config.compression);
    }

    #[test]
    fn test_duration_conversions() {
        let config = WebSocketConfig::default();
        assert_eq!(config.idle_timeout_duration().as_secs(), 300);
        assert_eq!(config.ping_interval_duration().as_secs(), 30);
    }
}
