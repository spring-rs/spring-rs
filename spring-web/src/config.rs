#[cfg(feature = "tls")]
use instant_acme::{LetsEncrypt, ZeroSsl};
use schemars::JsonSchema;
use serde::Deserialize;
use std::net::{IpAddr, Ipv4Addr};

/// spring-web Config
#[derive(Debug, JsonSchema, Deserialize)]
pub struct WebConfig {
    #[serde(default = "default_binding")]
    pub(crate) binding: IpAddr,
    #[serde(default = "default_port")]
    pub(crate) port: u16,
    pub(crate) middlewares: Option<Middlewares>,
    #[cfg(feature = "tls")]
    pub(crate) tls: TLS,
}

fn default_binding() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}

fn default_port() -> u16 {
    8080
}

/// Server middleware configuration structure.
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct Middlewares {
    /// Middleware that enable compression for the response.
    pub compression: Option<EnableMiddleware>,
    /// Middleware that limit the payload request.
    pub limit_payload: Option<LimitPayloadMiddleware>,
    /// Middleware that improve the tracing logger and adding trace id for each
    /// request.
    pub logger: Option<EnableMiddleware>,
    /// catch any code panic and log the error.
    pub catch_panic: Option<EnableMiddleware>,
    /// Setting a global timeout for the requests
    pub timeout_request: Option<TimeoutRequestMiddleware>,
    /// Setting cors configuration
    pub cors: Option<CorsMiddleware>,
    /// Serving static assets
    #[serde(rename = "static")]
    pub static_assets: Option<StaticAssetsMiddleware>,
}

/// Static asset middleware configuration
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct StaticAssetsMiddleware {
    /// toggle enable
    pub enable: bool,
    /// Check that assets must exist on disk
    #[serde(default = "bool::default")]
    pub must_exist: bool,
    /// Fallback page for a case when no asset exists (404). Useful for SPA
    /// (single page app) where routes are virtual.
    #[serde(default = "default_fallback")]
    pub fallback: String,
    /// Enable `precompressed_gzip`
    #[serde(default = "bool::default")]
    pub precompressed: bool,
    /// Uri for the assets
    #[serde(default = "default_assets_uri")]
    pub uri: String,
    /// Path for the assets
    #[serde(default = "default_assets_path")]
    pub path: String,
}

/// CORS middleware configuration
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct CorsMiddleware {
    /// toggle enable
    pub enable: bool,
    /// Allow origins
    pub allow_origins: Option<Vec<String>>,
    /// Allow headers
    pub allow_headers: Option<Vec<String>>,
    /// Allow methods
    pub allow_methods: Option<Vec<String>>,
    /// Max age
    pub max_age: Option<u64>,
}

/// Timeout middleware configuration
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct TimeoutRequestMiddleware {
    /// toggle enable
    pub enable: bool,
    /// Timeout request in milliseconds
    pub timeout: u64,
}

/// Limit payload size middleware configuration
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct LimitPayloadMiddleware {
    /// toggle enable
    pub enable: bool,
    /// Body limit. for example: 5mb
    pub body_limit: String,
}

/// A generic middleware configuration that can be enabled or
/// disabled.
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub struct EnableMiddleware {
    /// toggle enable
    pub enable: bool,
}

/// ssl/tls config
#[cfg(feature = "tls")]
#[derive(Debug, Clone, JsonSchema, Deserialize)]
pub enum TLS {
    AutoCert {
        #[serde(default = "default_acme_cert_dir")]
        acme_cert_dir: String,

        #[serde(default)]
        acme_server: AcmeServer,

        domain: String,

        contact: String,
    },
    ExistingCert {
        cert: String,
        pri_key: String,
    },
}

#[cfg(feature = "tls")]
#[derive(Debug, Default, Clone, JsonSchema, Deserialize)]
pub enum AcmeServer {
    /// https://letsencrypt.org/docs/
    #[default]
    LetsEncryptProduction,
    /// https://letsencrypt.org/docs/staging-environment/
    /// recommend testing against our staging environment before using our production environment.
    /// This will allow you to get things right before issuing trusted certificates and reduce the chance of your running up against rate limits.
    LetsEncryptStaging,
    /// https://zerossl.com/features/acme/
    ZeroSsl,
    /// * https://github.com/topics/acme-server
    /// * https://github.com/letsencrypt/boulder
    /// * https://github.com/smallstep/certificates
    Other(String),
}

#[cfg(feature = "tls")]
impl AcmeServer {
    pub(crate) fn url(&self) -> &str {
        match self {
            Self::LetsEncryptProduction => LetsEncrypt::Production.url(),
            Self::LetsEncryptStaging => LetsEncrypt::Staging.url(),
            Self::ZeroSsl => ZeroSsl::Production.url(),
            Self::Other(url) => url,
        }
    }
}

fn default_assets_path() -> String {
    "static".to_string()
}

fn default_assets_uri() -> String {
    "/static".to_string()
}

fn default_fallback() -> String {
    "index.html".to_string()
}

#[cfg(feature = "tls")]
fn default_acme_cert_dir() -> String {
    "./acme-cert".to_string()
}
