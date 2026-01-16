//! # spring-sa-token
//!
//! Automatic assembly for sa-token-rust.
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! spring-sa-token = { path = "../spring-sa-token" }  # Default: memory storage
//! # Or reuse spring-redis connection (recommended for production):
//! # spring-sa-token = { path = "../spring-sa-token", default-features = false, features = ["with-spring-redis", "with-web"] }
//! ```
//!
//! Configure in `config/app.toml`:
//!
//! ```toml
//! [sa-token]
//! token_name = "Authorization"
//! timeout = 86400
//! auto_renew = true
//! token_style = "uuid"
//! is_concurrent = true
//! ```
//!
//! Use in your application:
//!
//! ```rust,ignore
//! use spring::{auto_config, App};
//! use spring_web::{get, WebConfigurator, WebPlugin};
//! use spring_sa_token::{SaTokenPlugin, LoginIdExtractor, StpUtil};
//!
//! #[auto_config(WebConfigurator)]
//! #[tokio::main]
//! async fn main() {
//!     App::new()
//!         .add_plugin(SaTokenPlugin)
//!         .add_plugin(WebPlugin)
//!         .run()
//!         .await
//! }
//!
//! #[get("/user/info")]
//! async fn user_info(LoginIdExtractor(user_id): LoginIdExtractor) -> String {
//!     format!("Current user: {}", user_id)
//! }
//! ```

// ============================================================================
// Feature mutual exclusion checks
// Storage features are mutually exclusive: only one can be enabled at a time
// ============================================================================
#[cfg(all(feature = "with-spring-redis", feature = "memory"))]
compile_error!(
    "Features 'with-spring-redis' and 'memory' are mutually exclusive. \
     Choose one storage backend."
);

pub mod config;
pub mod configurator;
#[cfg(feature = "with-spring-redis")]
pub mod storage;

use crate::config::SaTokenConfig;
use sa_token_adapter::SaStorage;
use spring::app::AppBuilder;
use spring::async_trait;
use spring::config::ConfigRegistry;
use spring::plugin::ComponentRegistry;
use spring::plugin::{MutableComponentRegistry, Plugin};
#[cfg(feature = "with-web")]
use spring_web::LayerConfigurator;
use std::sync::Arc;

// ============================================================================
// Re-exports: Users only need to import spring-sa-token
// ============================================================================

// Re-export entire crates for full access
pub use sa_token_adapter;
pub use sa_token_core;
pub use sa_token_plugin_axum;

// Re-export config types to root
pub use crate::config::{CoreConfig, TokenStyle};

// Re-export commonly used types to root for convenience
pub use sa_token_core::{SaTokenManager, StpUtil};
pub use sa_token_plugin_axum::{LoginIdExtractor, OptionalSaTokenExtractor, SaTokenExtractor};
pub use sa_token_plugin_axum::{SaTokenLayer, SaTokenMiddleware, SaTokenState};

// Re-export procedural macros from spring-macros (WebError compatible)
pub use spring_macros::sa_check_login;
pub use spring_macros::sa_check_permission;
pub use spring_macros::sa_check_permissions_and;
pub use spring_macros::sa_check_permissions_or;
pub use spring_macros::sa_check_role;
pub use spring_macros::sa_check_roles_and;
pub use spring_macros::sa_check_roles_or;
pub use spring_macros::sa_ignore;

// Re-export storage implementations
#[cfg(feature = "memory")]
pub use sa_token_storage_memory::MemoryStorage;

// Re-export configurator types
pub use crate::configurator::{PathAuthBuilder, SaTokenAuthConfigurator, SaTokenConfigurator};

// Re-export error types
pub use sa_token_core::error::SaTokenError;
use sa_token_core::PathAuthConfig;

/// Sa-Token plugin for spring-rs
///
/// This plugin initializes the Sa-Token authentication system and registers
/// `SaTokenState` as a component that can be injected into handlers.
pub struct SaTokenPlugin;

#[async_trait]
impl Plugin for SaTokenPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<SaTokenConfig>()
            .expect("sa-token plugin config load failed");

        tracing::info!("Initializing Sa-Token plugin...");

        let state = Self::create_state(app, config)
            .await
            .expect("sa-token state creation failed");

        tracing::debug!(
            "SaTokenState manager.config.token_name = {}",
            state.manager.config.token_name
        );

        tracing::info!("Sa-Token plugin initialized successfully");

        // Register SaTokenState as a component
        app.add_component(state.clone());

        // Automatically register SaTokenLayer as a router layer
        // This middleware will extract tokens from requests and validate them
        #[cfg(feature = "with-web")]
        {
            // Get path-based authentication configuration from component
            if let Some(config) = app.get_component::<PathAuthConfig>() {
                tracing::info!("Registering SaTokenLayer with path-based authentication");
                app.add_router_layer(move |router| {
                    router.layer(SaTokenLayer::with_path_auth(state.clone(), config.clone()))
                });
            } else {
                tracing::info!("Registering SaTokenLayer as router middleware (no path config)");
                app.add_router_layer(move |router| router.layer(SaTokenLayer::new(state.clone())));
            }
        }
    }

    fn name(&self) -> &str {
        "spring_sa_token::SaTokenPlugin"
    }

    #[cfg(feature = "with-spring-redis")]
    fn dependencies(&self) -> Vec<&str> {
        vec!["spring_redis::RedisPlugin"]
    }
}

impl SaTokenPlugin {
    /// Create SaTokenState from configuration
    ///
    /// Uses SaTokenConfig::builder() from sa-token-core which supports most config fields
    /// and automatically initializes StpUtil.
    ///
    /// Note: The following fields are not supported by the builder (using defaults):
    /// - is_log, is_read_cookie, is_read_header, is_read_body
    async fn create_state(app: &AppBuilder, config: SaTokenConfig) -> anyhow::Result<SaTokenState> {
        // Configure storage based on features
        let storage = Self::configure_storage(app, &config).await?;

        tracing::debug!(
            "Sa-Token config: token_name={}, timeout={}, auto_renew={}, is_concurrent={}, is_share={}",
            config.token_name, config.timeout, config.auto_renew, config.is_concurrent, config.is_share
        );

        // Use SaTokenConfig::builder() from sa-token-core which supports more config fields
        // and automatically initializes StpUtil on build()
        let mut builder = sa_token_core::SaTokenConfig::builder()
            .storage(storage)
            .token_name(config.token_name)
            .timeout(config.timeout)
            .active_timeout(config.active_timeout)
            .auto_renew(config.auto_renew)
            .is_concurrent(config.is_concurrent)
            .is_share(config.is_share)
            .token_style(config.token_style)
            .enable_nonce(config.enable_nonce)
            .nonce_timeout(config.nonce_timeout)
            .enable_refresh_token(config.enable_refresh_token)
            .refresh_token_timeout(config.refresh_token_timeout);

        // Set optional fields
        if let Some(prefix) = config.token_prefix {
            builder = builder.token_prefix(prefix);
        }
        if let Some(key) = config.jwt_secret_key {
            builder = builder.jwt_secret_key(key);
        }
        if let Some(algorithm) = config.jwt_algorithm {
            builder = builder.jwt_algorithm(algorithm);
        }
        if let Some(issuer) = config.jwt_issuer {
            builder = builder.jwt_issuer(issuer);
        }
        if let Some(audience) = config.jwt_audience {
            builder = builder.jwt_audience(audience);
        }

        // build() creates SaTokenManager and auto-initializes StpUtil
        let manager = builder.build();

        // Create SaTokenState from manager
        Ok(SaTokenState::from_manager(manager))
    }

    /// Configure storage backend based on features and configuration
    ///
    /// Priority:
    /// 1. spring-redis component (if with-spring-redis feature enabled)
    /// 2. memory storage (if memory feature enabled)
    #[allow(unused_variables)]
    async fn configure_storage(
        app: &AppBuilder,
        config: &SaTokenConfig,
    ) -> anyhow::Result<Arc<dyn SaStorage>> {
        // Priority 1: Use spring-redis component if available
        #[cfg(feature = "with-spring-redis")]
        {
            if let Some(redis) = app.get_component::<spring_redis::Redis>() {
                tracing::info!("Using SpringRedisStorage (reusing spring-redis connection)");
                let storage = storage::SpringRedisStorage::new(redis);
                return Ok(Arc::new(storage));
            } else {
                anyhow::bail!(
                    "Feature 'with-spring-redis' is enabled but RedisPlugin is not added. \
                     Please add RedisPlugin before SaTokenPlugin."
                );
            }
        }

        // Priority 2: Fall back to memory storage
        #[cfg(feature = "memory")]
        {
            tracing::info!("Using Memory storage");
            return Ok(Arc::new(MemoryStorage::new()));
        }

        // No storage available
        #[cfg(not(any(feature = "memory", feature = "with-spring-redis")))]
        {
            anyhow::bail!(
                "No storage backend available. Enable 'memory' or 'with-spring-redis' feature."
            );
        }
    }
}
