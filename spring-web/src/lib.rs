//! spring-web
#[doc = include_str!("../README.md")]

/// spring-web config
pub mod config;
/// spring-web defined error
pub mod error;
/// axum extract
pub mod extractor;
/// axum route handler
pub mod handler;
use anyhow::Context;
pub use axum;
use config::{LimitPayloadMiddleware, Middlewares, StaticAssetsMiddleware, WebConfig, TLS};
pub use spring_boot::async_trait;
use spring_boot::config::Configurable;
use spring_boot::{
    app::{App, AppBuilder},
    error::Result,
    plugin::Plugin,
};
use std::{net::SocketAddr, ops::Deref, path::PathBuf, str::FromStr, sync::Arc, time::Duration};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

#[cfg(feature = "tls")]
mod acme;
#[cfg(feature = "tls")]
use axum_server::tls_rustls::RustlsConfig;

/// axum::routing::MethodFilter re-export
pub type MethodFilter = axum::routing::MethodFilter;
/// MethodRouter with AppState
pub type MethodRouter = axum::routing::MethodRouter<AppState>;
/// Router with AppState
pub type Router = axum::Router<AppState>;
/// Routers collection
pub type Routers = Vec<Router>;

/// Web Configurator
pub trait WebConfigurator {
    /// add route to app registry
    fn add_router(&mut self, router: Router) -> &mut Self;
}

impl WebConfigurator for AppBuilder {
    fn add_router(&mut self, router: Router) -> &mut Self {
        if let Some(routers) = self.get_component::<Routers>() {
            unsafe {
                let raw_ptr = Arc::into_raw(routers);
                let routers = &mut *(raw_ptr as *mut Vec<Router>);
                routers.push(router);
            }
            self
        } else {
            self.add_component(vec![router])
        }
    }
}

/// State of App
#[derive(Clone)]
pub struct AppState {
    /// App Registry Ref
    pub app: Arc<App>,
}

/// Web Plugin Definition
#[derive(Configurable)]
#[config_prefix = "web"]
pub struct WebPlugin;

#[async_trait]
impl Plugin for WebPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<WebConfig>(self)
            .expect("web plugin config load failed");

        // 1. collect router
        let routers = app.get_component::<Routers>();
        let mut router: Router = match routers {
            Some(rs) => {
                let mut router = Router::new();
                for r in rs.deref().iter() {
                    router = router.merge(r.to_owned());
                }
                router
            }
            None => Router::new(),
        };
        if let Some(middlewares) = &config.middlewares {
            router = Self::apply_middleware(router, middlewares);
        }
        #[cfg(feature = "tls")]
        app.add_scheduler(move |app: Arc<App>| Box::new(Self::schedule_tls(config, router, app)));
        #[cfg(not(feature = "tls"))]
        app.add_scheduler(move |app: Arc<App>| Box::new(Self::schedule(config, router, app)));
    }
}

impl WebPlugin {
    #[cfg(not(feature = "tls"))]
    async fn schedule(config: WebConfig, router: Router, app: Arc<App>) -> Result<String> {
        // bind tcp listener
        let addr = SocketAddr::from((config.binding, config.port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("bind tcp listener failed:{}", addr))?;
        tracing::info!("bind tcp listener: {}", addr);

        // start axum server
        let router = router.with_state(AppState { app });

        tracing::info!("axum server started");
        axum::serve(listener, router)
            .await
            .context("start axum server failed")?;

        Ok("axum schedule finished".to_string())
    }

    #[cfg(feature = "tls")]
    async fn schedule_tls(config: WebConfig, router: Router, app: Arc<App>) -> Result<String> {
        let addr = SocketAddr::from((config.binding, config.port));

        let router = router.with_state(AppState { app });

        match config.tls {
            TLS::ExistingCert { cert, pri_key } => {
                let config = RustlsConfig::from_pem_file(cert, pri_key)
                    .await
                    .context("read ssl pem file failed")?;
                axum_server::tls_rustls::bind_rustls(addr, config)
                    .serve(router.into_make_service())
                    .await
                    .context("start tls server failed")?;
            }
            TLS::AutoCert {} => {}
        }
        todo!()
    }

    fn apply_middleware(router: Router, middleware: &Middlewares) -> Router {
        let mut router = router;
        if let Some(catch_panic) = &middleware.catch_panic {
            if catch_panic.enable {
                router = router.layer(CatchPanicLayer::new());
            }
        }
        if let Some(compression) = &middleware.compression {
            if compression.enable {
                router = router.layer(CompressionLayer::new());
            }
        }
        if let Some(cors) = &middleware.cors {
            if cors.enable {
                let cors =
                    Self::build_cors_middleware(&cors).expect("cors middleware build failed");
                router = router.layer(cors);
            }
        }
        if let Some(limit_payload) = &middleware.limit_payload {
            if limit_payload.enable {
                let limit_payload = Self::build_body_limit_middleware(limit_payload)
                    .expect("limit payload middleware build failed");
                router = router.layer(limit_payload);
            }
        }
        if let Some(logger) = &middleware.logger {
            if logger.enable {
                router = router.layer(TraceLayer::new_for_http());
            }
        }
        if let Some(timeout_request) = &middleware.timeout_request {
            if timeout_request.enable {
                router = router.layer(TimeoutLayer::new(Duration::from_millis(
                    timeout_request.timeout,
                )));
            }
        }

        if let Some(static_assets) = &middleware.static_assets {
            if static_assets.enable {
                return Self::apply_static_dir(router, static_assets);
            }
        }
        router
    }

    fn apply_static_dir(router: Router, static_assets: &StaticAssetsMiddleware) -> Router {
        if static_assets.must_exist
            && (!PathBuf::from(&static_assets.path).exists()
                || !PathBuf::from(&static_assets.fallback).exists())
        {
            panic!(
                "one of the static path are not found, Folder `{}` fallback: `{}`",
                static_assets.path, static_assets.fallback
            );
        }

        let serve_dir = ServeDir::new(static_assets.path.clone())
            .not_found_service(ServeFile::new(static_assets.fallback.clone()));

        let service = if static_assets.precompressed {
            tracing::info!("[Middleware] Enable precompressed static assets");
            serve_dir.precompressed_gzip()
        } else {
            serve_dir
        };

        router.nest_service(&static_assets.uri, service)
    }

    fn build_cors_middleware(cors: &config::CorsMiddleware) -> Result<CorsLayer> {
        let mut layer = CorsLayer::new();

        if let Some(allow_origins) = &cors.allow_origins {
            // testing CORS, assuming https://example.com in the allow list:
            // $ curl -v --request OPTIONS 'localhost:5150/api/_ping' -H 'Origin: https://example.com' -H 'Access-Control-Request-Method: GET'
            // look for '< access-control-allow-origin: https://example.com' in response.
            // if it doesn't appear (test with a bogus domain), it is not allowed.
            let mut origins = Vec::with_capacity(allow_origins.len());
            for origin in allow_origins {
                let origin = origin
                    .parse()
                    .with_context(|| format!("cors origin parse failed:{}", origin))?;
                origins.push(origin);
            }
            layer = layer.allow_origin(origins);
        }

        if let Some(allow_headers) = &cors.allow_headers {
            let mut headers = Vec::with_capacity(allow_headers.len());
            for header in allow_headers {
                let header = header
                    .parse()
                    .with_context(|| format!("http header parse failed:{}", header))?;
                headers.push(header);
            }
            layer = layer.allow_headers(headers);
        }

        if let Some(allow_methods) = &cors.allow_methods {
            let mut methods = Vec::with_capacity(allow_methods.len());
            for method in allow_methods {
                let method = method
                    .parse()
                    .with_context(|| format!("http method parse failed:{}", method))?;
                methods.push(method);
            }
            layer = layer.allow_methods(methods);
        }

        if let Some(max_age) = cors.max_age {
            layer = layer.max_age(Duration::from_secs(max_age));
        }

        Ok(layer)
    }

    fn build_body_limit_middleware(
        limit_payload: &LimitPayloadMiddleware,
    ) -> Result<RequestBodyLimitLayer> {
        let limit = byte_unit::Byte::from_str(&limit_payload.body_limit).with_context(|| {
            format!(
                "parse limit payload str failed: {}",
                &limit_payload.body_limit
            )
        })?;

        Ok(RequestBodyLimitLayer::new(limit.as_u64() as usize))
    }
}
