[![crates.io](https://img.shields.io/crates/v/spring-sa-token.svg)](https://crates.io/crates/spring-sa-token)
[![Documentation](https://docs.rs/spring-sa-token/badge.svg)](https://docs.rs/spring-sa-token)

`spring-sa-token` is an automatic assembly for [sa-token-rust](https://github.com/click33/sa-token-rust).

## Dependencies

```toml
# Default: memory storage (for development)
spring-sa-token = { version = "<version>" }

# Production: reuse spring-redis connection (recommended)
spring-sa-token = { version = "<version>", default-features = false, features = ["with-spring-redis", "with-web"] }
```

Optional **features**:
* `memory`: In-memory storage (default, for development/testing)
* `with-spring-redis`: Use spring-redis connection pool for storage (recommended)
* `with-web`: Enable axum web integration (middleware, extractors)

## Configuration items
For detailed documentation and configuration, see [sa-token-rust docs](https://github.com/click33/sa-token-rust/tree/main/docs)

```toml
[sa-token]
# Token name (key in header or cookie)
token_name = "Authorization"

# Token timeout in seconds, -1 means permanent
# Default: 2592000 (30 days)
timeout = 86400

# Token active timeout in seconds, -1 means no limit
# If no requests within this time, token becomes invalid
active_timeout = 3600

# Enable auto renew - automatically refresh token on each request
auto_renew = true

# Allow concurrent login for same account
is_concurrent = true

# Share token when multiple logins for same account
is_share = true

# Token style: Uuid, SimpleUuid, Random32, Random64, Random128, Jwt
token_style = "Uuid"

# Token prefix (e.g., "Bearer ")
token_prefix = "Bearer "

# JWT configuration (only when token_style = "Jwt")
jwt_secret_key = "your-secret-key"
jwt_algorithm = "HS256"    # HS256, HS384, HS512
jwt_issuer = "my-app"
jwt_audience = "my-users"

# Enable nonce for replay attack prevention
enable_nonce = false
nonce_timeout = 300

# Enable refresh token
enable_refresh_token = false
refresh_token_timeout = 604800  # 7 days
```

## Quick Start

### 1. Add plugins to your application

```rust
use spring::{auto_config, App};
use spring_redis::RedisPlugin;
use spring_sa_token::{SaTokenPlugin, SaTokenAuthConfigurator};
use spring_web::{WebPlugin, WebConfigurator};

mod security;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(RedisPlugin)       // Required for with-spring-redis feature
        .add_plugin(SaTokenPlugin)
        .add_plugin(WebPlugin)
        .sa_token_auth(security::SecurityConfig)  // Configure path-based auth
        .run()
        .await
}
```

### 2. Define security configuration

Create `src/security.rs`:

```rust
use spring_sa_token::{PathAuthBuilder, SaTokenConfigurator};

pub struct SecurityConfig;

impl SaTokenConfigurator for SecurityConfig {
    fn configure(&self, auth: PathAuthBuilder) -> PathAuthBuilder {
        auth
            // Paths requiring authentication
            .include("/user/**")
            .include("/admin/**")
            .include("/api/**")
            // Public paths (no auth required)
            .exclude("/login")
            .exclude("/api/health")
    }
}
```

### 3. Implement login endpoint

```rust
use spring_sa_token::StpUtil;
use spring_web::{post, axum::response::IntoResponse, extractor::Json, error::Result};

#[post("/login")]
async fn login(Json(req): Json<LoginRequest>) -> Result<impl IntoResponse> {
    // Validate credentials (your business logic)
    if req.username == "admin" && req.password == "123456" {
        // Login and get token
        let token = StpUtil::login(&req.username).await?;

        // Optionally set roles and permissions
        StpUtil::set_roles(&req.username, vec!["admin".to_string()]).await?;
        StpUtil::set_permissions(&req.username, vec!["user:list".to_string()]).await?;

        Ok(Json(LoginResponse {
            token: token.as_str().to_string(),
            message: "Login successful".to_string(),
        }))
    } else {
        Ok(Json(ErrorResponse { message: "Invalid credentials".to_string() }))
    }
}
```

### 4. Access protected routes

```rust
use spring_sa_token::LoginIdExtractor;
use spring_web::{get, axum::response::IntoResponse, extractor::Json, error::Result};

#[get("/user/info")]
async fn user_info(LoginIdExtractor(user_id): LoginIdExtractor) -> Result<impl IntoResponse> {
    // user_id is automatically extracted from the token
    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "message": format!("Hello, {}!", user_id)
    })))
}
```

## Procedural Macros

`spring-sa-token` provides several procedural macros for declarative security:

### `#[sa_check_login]`

Verify user is logged in:

```rust
#[get("/api/profile")]
#[sa_check_login]
async fn get_profile(LoginIdExtractor(user_id): LoginIdExtractor) -> Result<impl IntoResponse> {
    Ok(Json(serde_json::json!({ "user_id": user_id })))
}
```

### `#[sa_check_role("role")]`

Verify user has specific role:

```rust
#[get("/admin/dashboard")]
#[sa_check_role("admin")]
async fn admin_dashboard() -> impl IntoResponse {
    "Welcome to admin dashboard"
}
```

### `#[sa_check_roles_and("role1", "role2")]`

Verify user has ALL specified roles:

```rust
#[get("/api/super-admin")]
#[sa_check_roles_and("admin", "super")]
async fn super_admin_only() -> impl IntoResponse {
    "You have both admin and super roles"
}
```

### `#[sa_check_roles_or("role1", "role2")]`

Verify user has ANY of the specified roles:

```rust
#[get("/api/management")]
#[sa_check_roles_or("admin", "manager")]
async fn management_area() -> impl IntoResponse {
    "You have admin or manager role"
}
```

### `#[sa_check_permission("permission")]`

Verify user has specific permission:

```rust
#[get("/admin/users")]
#[sa_check_permission("user:list")]
async fn list_users() -> impl IntoResponse {
    "User list"
}
```

### `#[sa_check_permissions_and("perm1", "perm2")]`

Verify user has ALL specified permissions:

```rust
#[post("/api/user/batch-modify")]
#[sa_check_permissions_and("user:edit", "user:delete")]
async fn batch_modify() -> impl IntoResponse {
    "Batch modify successful"
}
```

### `#[sa_check_permissions_or("perm1", "perm2")]`

Verify user has ANY of the specified permissions:

```rust
#[post("/api/user/create-or-update")]
#[sa_check_permissions_or("user:add", "user:edit")]
async fn create_or_update() -> impl IntoResponse {
    "Create or update successful"
}
```

### `#[sa_ignore]`

Skip authentication for specific endpoint (even if path matches include rules):

```rust
#[get("/api/health")]
#[sa_ignore]
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok" }))
}
```

## StpUtil API

The `StpUtil` struct provides static methods for token operations:

### Login/Logout

```rust
// Login and get token
let token = StpUtil::login("user_id").await?;

// Logout current token
StpUtil::logout("token").await?;

// Logout by login ID (invalidate all tokens)
StpUtil::logout_by_login_id("user_id").await?;

// Check if user is logged in
let is_login = StpUtil::is_login_by_login_id("user_id").await;
```

### Token Operations

```rust
// Get token by login ID
let token = StpUtil::get_token_by_login_id("user_id").await;

// Get login ID by token
let login_id = StpUtil::get_login_id_by_token("token").await;
```

### Roles and Permissions

```rust
// Set roles
StpUtil::set_roles("user_id", vec!["admin".to_string(), "user".to_string()]).await?;

// Get roles
let roles = StpUtil::get_roles("user_id").await;

// Check role
let has_role = StpUtil::has_role("user_id", "admin").await;

// Set permissions
StpUtil::set_permissions("user_id", vec!["user:list".to_string()]).await?;

// Get permissions
let permissions = StpUtil::get_permissions("user_id").await;

// Check permission
let has_perm = StpUtil::has_permission("user_id", "user:list").await;
```

## Extractors

### `LoginIdExtractor`

Extract current user's login ID from request:

```rust
use spring_sa_token::LoginIdExtractor;

#[get("/user/info")]
async fn user_info(LoginIdExtractor(user_id): LoginIdExtractor) -> impl IntoResponse {
    format!("Current user: {}", user_id)
}
```

### `OptionalSaTokenExtractor`

Extract token info optionally (returns None if not authenticated):

```rust
use spring_sa_token::OptionalSaTokenExtractor;

#[get("/public")]
async fn public_endpoint(token: OptionalSaTokenExtractor) -> impl IntoResponse {
    match token.0 {
        Some(info) => format!("Logged in as: {}", info.login_id),
        None => "Not logged in".to_string(),
    }
}
```

### `SaTokenExtractor`

Extract full token info (fails if not authenticated):

```rust
use spring_sa_token::SaTokenExtractor;

#[get("/protected")]
async fn protected_endpoint(SaTokenExtractor(info): SaTokenExtractor) -> impl IntoResponse {
    format!("Token: {}, User: {}", info.token, info.login_id)
}
```

## Component Access

Access `SaTokenState` component for advanced operations:

```rust
use spring_sa_token::SaTokenState;
use spring_web::extractor::Component;

#[get("/api/config")]
async fn get_config(Component(state): Component<SaTokenState>) -> impl IntoResponse {
    let config = &state.manager.config;
    Json(serde_json::json!({
        "token_name": config.token_name,
        "timeout": config.timeout,
        "token_style": format!("{:?}", config.token_style),
    }))
}
```

## Error Handling

All security macros return `spring_web::error::WebError` on failure, which can be handled by your error handling middleware:

```rust
use spring_web::error::Result;

#[get("/admin/dashboard")]
#[sa_check_role("admin")]
async fn admin_dashboard() -> Result<impl IntoResponse> {
    // If user doesn't have "admin" role, returns 403 Forbidden
    Ok(Json(serde_json::json!({ "message": "Welcome!" })))
}
```

Complete code reference [`sa-token-example`][sa-token-example]

[sa-token-example]: https://github.com/spring-rs/spring-rs/tree/master/examples/sa-token-example