//! Security configuration module
//!
//! This module defines path-based authentication rules for the application.

use spring_sa_token::{PathAuthBuilder, SaTokenConfigurator};

/// Security configuration class
///
/// Implements `SaTokenConfigurator` to define which paths require authentication.
pub struct SecurityConfig;

impl SaTokenConfigurator for SecurityConfig {
    fn configure(&self, auth: PathAuthBuilder) -> PathAuthBuilder {
        auth
            // All paths under /user/**, /admin/**, /api/** require authentication
            .include("/user/**")
            .include("/admin/**")
            .include("/api/**")
            // These paths are public (no auth required)
            .exclude("/login")
            .exclude("/api/health") // Health check is public
            .exclude("/api/config") // Config check is public (for debugging)
    }
}