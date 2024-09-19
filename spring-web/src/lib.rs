//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io/docs/plugins/spring-web)
#![doc = include_str!("../README.md")]

/// spring-web config
pub mod config;
/// spring-web defined error
pub mod error;
/// axum extract
pub mod extractor;
/// axum route handler
pub mod handler;

pub use axum;
use axum::Extension;
pub use spring::async_trait;
/////////////////web-macros/////////////////////
/// To use these Procedural Macros, you need to add `spring-web` dependency
pub use spring_macros::delete;
pub use spring_macros::get;
pub use spring_macros::head;
pub use spring_macros::nest;
pub use spring_macros::options;
pub use spring_macros::patch;
pub use spring_macros::post;
pub use spring_macros::put;
pub use spring_macros::route;
pub use spring_macros::routes;
pub use spring_macros::trace;

use anyhow::Context;
use config::{
    EnableMiddleware, LimitPayloadMiddleware, Middlewares, StaticAssetsMiddleware,
    TimeoutRequestMiddleware, WebConfig,
};
use spring::{
    app::{App, AppBuilder},
    config::ConfigRegistry,
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

/// axum::routing::MethodFilter re-export
pub type MethodFilter = axum::routing::MethodFilter;
/// MethodRouter with AppState
pub type MethodRouter = axum::routing::MethodRouter;
/// Router with AppState
pub type Router = axum::Router;
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
pub struct WebPlugin;

#[async_trait]
impl Plugin for WebPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<WebConfig>()
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
        if let Some(middlewares) = config.middlewares {
            router = Self::apply_middleware(router, middlewares);
        }

        let addr = SocketAddr::from((config.binding, config.port));

        app.add_scheduler(move |app: Arc<App>| Box::new(Self::schedule(addr, router, app)));
    }
}

impl WebPlugin {
    async fn schedule(addr: SocketAddr, router: Router, app: Arc<App>) -> Result<String> {
        // 2. bind tcp listener
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("bind tcp listener failed:{}", addr))?;
        tracing::info!("bind tcp listener: {}", addr);

        // 3. axum server
        let state = AppState { app };
        let router = router.layer(Extension(state));

        tracing::info!("axum server started");
        axum::serve(listener, router)
            .await
            .context("start axum server failed")?;

        Ok("axum schedule finished".to_string())
    }

    fn apply_middleware(mut router: Router, middleware: Middlewares) -> Router {
        if Some(EnableMiddleware { enable: true }) == middleware.catch_panic {
            router = router.layer(CatchPanicLayer::new());
        }
        if Some(EnableMiddleware { enable: true }) == middleware.compression {
            router = router.layer(CompressionLayer::new());
        }
        if Some(EnableMiddleware { enable: true }) == middleware.logger {
            router = router.layer(TraceLayer::new_for_http());
        }
        if let Some(TimeoutRequestMiddleware { enable, timeout }) = middleware.timeout_request {
            if enable {
                router = router.layer(TimeoutLayer::new(Duration::from_millis(timeout)));
            }
        }
        if let Some(LimitPayloadMiddleware { enable, body_limit }) = middleware.limit_payload {
            if enable {
                let limit = byte_unit::Byte::from_str(&body_limit)
                    .unwrap_or_else(|_| panic!("parse limit payload str failed: {}", &body_limit));

                let limit_payload = RequestBodyLimitLayer::new(limit.as_u64() as usize);
                router = router.layer(limit_payload);
            }
        }
        if let Some(cors) = middleware.cors {
            if cors.enable {
                let cors =
                    Self::build_cors_middleware(&cors).expect("cors middleware build failed");
                router = router.layer(cors);
            }
        }
        if let Some(static_assets) = middleware.static_assets {
            if static_assets.enable {
                router = Self::apply_static_dir(router, static_assets);
            }
        }
        router
    }

    fn apply_static_dir(router: Router, static_assets: StaticAssetsMiddleware) -> Router {
        if static_assets.must_exist
            && (!PathBuf::from(&static_assets.path).exists()
                || !PathBuf::from(&static_assets.fallback).exists())
        {
            panic!(
                "one of the static path are not found, Folder `{}` fallback: `{}`",
                static_assets.path, static_assets.fallback
            );
        }

        let serve_dir = ServeDir::new(static_assets.path)
            .not_found_service(ServeFile::new(static_assets.fallback));

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
}
