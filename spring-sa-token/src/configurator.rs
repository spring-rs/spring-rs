//! Sa-Token Configurator module
//!
//! This module provides a Spring Security-like configuration mechanism
//! for path-based authentication.
//!
//! # Example
//!
//! Define your security configuration in a separate file:
//!
//! ```rust,ignore
//! // security.rs
//! use spring_sa_token::{PathAuthBuilder, SaTokenConfigurator};
//!
//! pub struct SecurityConfig;
//!
//! impl SaTokenConfigurator for SecurityConfig {
//!     fn configure(&self, auth: PathAuthBuilder) -> PathAuthBuilder {
//!         auth.include("/user/**")
//!             .include("/admin/**")
//!             .exclude("/login")
//!             .exclude("/public/**")
//!     }
//! }
//! ```
//!
//! Then use it in main.rs:
//!
//! ```rust,ignore
//! // main.rs
//! mod security;
//! use spring_sa_token::SaTokenAuthConfigurator;
//!
//! App::new()
//!     .add_plugin(RedisPlugin)
//!     .add_plugin(SaTokenPlugin)
//!     .add_plugin(WebPlugin)
//!     .sa_token_auth(security::SecurityConfig)
//!     .run()
//!     .await
//! ```

use sa_token_core::router::PathAuthConfig;
use spring::app::AppBuilder;
use spring::plugin::MutableComponentRegistry;

/// Trait for configuring Sa-Token path-based authentication
///
/// Implement this trait to define your security configuration,
/// similar to Spring Security's configuration classes.
///
/// # Example
///
/// ```rust,ignore
/// use spring_sa_token::{PathAuthBuilder, SaTokenConfigurator};
///
/// pub struct SecurityConfig;
///
/// impl SaTokenConfigurator for SecurityConfig {
///     fn configure(&self, auth: PathAuthBuilder) -> PathAuthBuilder {
///         auth.include("/api/**")
///             .include("/user/**")
///             .exclude("/login")
///             .exclude("/public/**")
///     }
/// }
/// ```
pub trait SaTokenConfigurator: Send + Sync + 'static {
    /// Configure path-based authentication rules
    ///
    /// Receives a PathAuthBuilder and should return it with your rules applied.
    fn configure(&self, auth: PathAuthBuilder) -> PathAuthBuilder;
}

/// Builder for path-based authentication configuration
///
/// Provides a fluent API for configuring which paths require authentication.
///
/// # Supported Patterns (Ant-style)
///
/// - `/**` - Match all paths
/// - `/api/**` - Match all paths starting with `/api/`
/// - `/api/*` - Match single-level paths under `/api/` (not nested)
/// - `*.html` - Match paths ending with `.html`
/// - `/exact` - Exact match
///
/// # Example
///
/// ```rust,ignore
/// let config = PathAuthBuilder::new()
///     .include("/api/**")
///     .include("/user/**")
///     .exclude("/login")
///     .exclude("/register")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
pub struct PathAuthBuilder {
    pub include: Vec<String>,
    pub exclude: Vec<String>,
}

impl PathAuthBuilder {
    /// Create a new PathAuthBuilder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a path pattern that requires authentication
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// auth.include("/api/**")  // All paths under /api/ require auth
    /// ```
    pub fn include(mut self, pattern: impl Into<String>) -> Self {
        self.include.push(pattern.into());
        self
    }

    /// Add multiple path patterns that require authentication
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// auth.include_all(["/api/**", "/user/**", "/admin/**"])
    /// ```
    pub fn include_all(mut self, patterns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.include.extend(patterns.into_iter().map(|p| p.into()));
        self
    }

    /// Add a path pattern that is excluded from authentication
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// auth.exclude("/login")  // /login doesn't require auth
    /// ```
    pub fn exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude.push(pattern.into());
        self
    }

    /// Add multiple path patterns that are excluded from authentication
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// auth.exclude_all(["/login", "/register", "/public/**"])
    /// ```
    pub fn exclude_all(mut self, patterns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.exclude.extend(patterns.into_iter().map(|p| p.into()));
        self
    }

    /// Alias for `exclude` - paths that don't require authentication (permit all)
    ///
    /// Named similarly to Spring Security's `permitAll()`
    pub fn permit_all(self, pattern: impl Into<String>) -> Self {
        self.exclude(pattern)
    }

    /// Alias for `include` - paths that require authentication
    ///
    /// Named similarly to Spring Security's `authenticated()`
    pub fn authenticated(self, pattern: impl Into<String>) -> Self {
        self.include(pattern)
    }

    /// Check if any include patterns are configured
    pub fn is_configured(&self) -> bool {
        !self.include.is_empty()
    }

    /// Build the final PathAuthConfig
    pub fn build(self) -> PathAuthConfig {
        PathAuthConfig::new()
            .include(self.include)
            .exclude(self.exclude)
    }

    /// Merge another builder's rules into this one
    pub fn merge(mut self, other: PathAuthBuilder) -> Self {
        self.include.extend(other.include);
        self.exclude.extend(other.exclude);
        self
    }
}

// ============================================================================
// SaTokenAuthConfigurator - Fluent API for configuring path-based auth
// ============================================================================

/// Trait for configuring Sa-Token path-based authentication via AppBuilder
///
/// This provides a fluent API to configure authentication directly on the App.
///
/// # Example
///
/// Define your security configuration in a separate file:
///
/// ```rust,ignore
/// // security.rs
/// use spring_sa_token::{PathAuthBuilder, SaTokenConfigurator};
///
/// pub struct SecurityConfig;
///
/// impl SaTokenConfigurator for SecurityConfig {
///     fn configure(&self, auth: PathAuthBuilder) -> PathAuthBuilder {
///         auth.include("/user/**")
///             .include("/admin/**")
///             .include("/api/**")
///             .exclude("/")
///             .exclude("/login")
///             .exclude("/public/**")
///             .exclude("/api/health")
///     }
/// }
/// ```
///
/// Then use it in main.rs:
///
/// ```rust,ignore
/// // main.rs
/// mod security;
/// use spring_sa_token::SaTokenAuthConfigurator;
///
/// App::new()
///     .add_plugin(RedisPlugin)
///     .add_plugin(SaTokenPlugin)
///     .add_plugin(WebPlugin)
///     .sa_token_auth(security::SecurityConfig)
///     .run()
///     .await
/// ```
pub trait SaTokenAuthConfigurator {
    /// Configure path-based authentication rules using a SaTokenConfigurator implementation
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// app.sa_token_auth(SecurityConfig);
    /// ```
    fn sa_token_auth<C>(&mut self, configurator: C) -> &mut Self
    where
        C: SaTokenConfigurator;
}

impl SaTokenAuthConfigurator for AppBuilder {
    fn sa_token_auth<C>(&mut self, configurator: C) -> &mut Self
    where
        C: SaTokenConfigurator ,
    {
        let builder = configurator.configure(PathAuthBuilder::new());
        if builder.is_configured() {
            let config = builder.build();
            self.add_component(config)
        } else {
            self
        }
    }
}

impl SaTokenConfigurator for PathAuthBuilder {
    fn configure(&self, auth: PathAuthBuilder) -> PathAuthBuilder {
        auth.merge(self.clone())
    }
}
