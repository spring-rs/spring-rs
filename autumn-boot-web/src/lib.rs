pub mod config;

use anyhow::Context;
use async_trait::async_trait;
use autumn_boot::{app::App, error::Result, plugin::Plugin};
use axum::Router;
use config::{LimitPayloadMiddleware, Middlewares, StaticAssetsMiddleware, WebConfig};
use std::{net::SocketAddr, path::PathBuf, str::FromStr, time::Duration};
use tokio::net::TcpListener;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::CorsLayer,
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

pub struct WebPlugin;

#[async_trait]
impl Plugin for WebPlugin {
    async fn build(&self, app: &mut App) {
        let config = app
            .get_config::<WebConfig>(self)
            .context(format!("web plugin config load failed"))
            .expect("axum web plugin load failed");

        let (listener, router) = Self::serve(config)
            .await
            .expect("axum web plugin load failed");

        // 3. axum server
        app.add_scheduler(tokio::spawn(async {
            tracing::info!("axum server started");
            axum::serve(listener, router.into_make_service())
                .await
                .context("start axum server failed")?;

            Ok("axum schedule finished".to_string())
        }));
    }

    fn config_prefix(&self) -> &str {
        "web"
    }
}

impl WebPlugin {
    async fn serve(config: WebConfig) -> Result<(TcpListener, Router)> {
        let addr = SocketAddr::from((config.binding, config.port));

        // 1. collect router
        let mut router = Router::new(); // TODO
        if let Some(middlewares) = config.middlewares {
            router = Self::apply_middleware(router, middlewares);
        }
        // todo: State

        // 2. bind tcp listener
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("bind tcp listener failed:{}", addr))?;

        tracing::info!("bind tcp listener: {}", addr);

        Ok((listener, router))
    }

    fn apply_middleware(router: Router, middleware: Middlewares) -> Router {
        let mut router = router;
        if let Some(catch_panic) = middleware.catch_panic {
            if catch_panic.enable {
                router = router.layer(CatchPanicLayer::new());
            }
        }
        if let Some(compression) = middleware.compression {
            if compression.enable {
                router = router.layer(CompressionLayer::new());
            }
        }
        if let Some(cors) = middleware.cors {
            if cors.enable {
                let cors =
                    Self::build_cors_middleware(&cors).expect("cors middleware build failed");
                router = router.layer(cors);
            }
        }
        if let Some(limit_payload) = middleware.limit_payload {
            if limit_payload.enable {
                let limit_payload = Self::build_body_limit_middleware(limit_payload)
                    .expect("limit payload middleware build failed");
                router = router.layer(limit_payload);
            }
        }
        if let Some(logger) = middleware.logger {
            if logger.enable {
                router = router.layer(TraceLayer::new_for_http());
            }
        }
        if let Some(timeout_request) = middleware.timeout_request {
            if timeout_request.enable {
                router = router.layer(TimeoutLayer::new(Duration::from_millis(
                    timeout_request.timeout,
                )));
            }
        }

        if let Some(static_assets) = middleware.static_assets {
            if static_assets.enable {
                return Self::apply_static_dir(router, static_assets);
            }
        }
        router
    }

    fn apply_static_dir(router: Router, static_assets: StaticAssetsMiddleware) -> Router {
        if static_assets.must_exist
            && (!PathBuf::from(&static_assets.folder.path).exists()
                || !PathBuf::from(&static_assets.fallback).exists())
        {
            panic!(
                "one of the static path are not found, Folder `{}` fallback: `{}`",
                static_assets.folder.path, static_assets.fallback
            );
        }

        let serve_dir = ServeDir::new(static_assets.folder.path)
            .not_found_service(ServeFile::new(static_assets.fallback));

        let service = if static_assets.precompressed {
            tracing::info!("[Middleware] Enable precompressed static assets");
            serve_dir.precompressed_gzip()
        } else {
            serve_dir
        };

        router.nest_service(&static_assets.folder.uri, service)
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
        limit_payload: LimitPayloadMiddleware,
    ) -> Result<RequestBodyLimitLayer> {
        let limit =
            byte_unit::Byte::from_str(&limit_payload.body_limit).with_context(|| format!(""))?;

        Ok(RequestBodyLimitLayer::new(limit.as_u64() as usize))
    }
}
