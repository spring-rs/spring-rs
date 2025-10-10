//! [![spring-rs](https://img.shields.io/github/stars/spring-rs/spring-rs)](https://spring-rs.github.io/docs/plugins/spring-web)
#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://spring-rs.github.io/favicon.ico")]
#![doc(html_logo_url = "https://spring-rs.github.io/logo.svg")]

/// spring-web config
pub mod config;
/// spring-web defined error
pub mod error;
/// axum extract
pub mod extractor;
/// axum route handler
pub mod handler;
pub mod middleware;
#[cfg(feature = "openapi")]
pub mod openapi;

#[cfg(feature = "socket_io")]
pub use { socketioxide, rmpv };

pub use axum;
pub use spring::async_trait;
/////////////////web-macros/////////////////////
/// To use these Procedural Macros, you need to add `spring-web` dependency
pub use spring_macros::middlewares;
pub use spring_macros::nest;

// route macros
pub use spring_macros::delete;
pub use spring_macros::get;
pub use spring_macros::head;
pub use spring_macros::options;
pub use spring_macros::patch;
pub use spring_macros::post;
pub use spring_macros::put;
pub use spring_macros::route;
pub use spring_macros::routes;
pub use spring_macros::trace;

/// SocketIO macros
#[cfg(feature = "socket_io")]
pub use spring_macros::on_connection;
#[cfg(feature = "socket_io")]
pub use spring_macros::on_disconnect;
#[cfg(feature = "socket_io")]
pub use spring_macros::on_fallback;
#[cfg(feature = "socket_io")]
pub use spring_macros::subscribe_message;

/// OpenAPI macros
#[cfg(feature = "openapi")]
pub use spring_macros::api_route;
#[cfg(feature = "openapi")]
pub use spring_macros::api_routes;
#[cfg(feature = "openapi")]
pub use spring_macros::delete_api;
#[cfg(feature = "openapi")]
pub use spring_macros::get_api;
#[cfg(feature = "openapi")]
pub use spring_macros::head_api;
#[cfg(feature = "openapi")]
pub use spring_macros::options_api;
#[cfg(feature = "openapi")]
pub use spring_macros::patch_api;
#[cfg(feature = "openapi")]
pub use spring_macros::post_api;
#[cfg(feature = "openapi")]
pub use spring_macros::put_api;
#[cfg(feature = "openapi")]
pub use spring_macros::trace_api;

/// axum::routing::MethodFilter re-export
pub use axum::routing::MethodFilter;

/// Router with AppState
#[cfg(not(feature = "openapi"))]
pub type Router = axum::Router;
/// MethodRouter with AppState
pub use axum::routing::MethodRouter;

#[cfg(feature = "openapi")]
pub use aide;
#[cfg(feature = "openapi")]
pub use aide::openapi::OpenApi;
#[cfg(feature = "openapi")]
pub type Router = aide::axum::ApiRouter;
#[cfg(feature = "openapi")]
pub use aide::axum::routing::ApiMethodRouter;

#[cfg(feature = "openapi")]
use aide::transform::TransformOpenApi;

use anyhow::Context;
use axum::Extension;
use config::ServerConfig;
use config::WebConfig;
use spring::plugin::component::ComponentRef;
use spring::plugin::ComponentRegistry;
use spring::plugin::MutableComponentRegistry;
use spring::{
    app::{App, AppBuilder},
    config::ConfigRegistry,
    error::Result,
    plugin::Plugin,
};
use std::{net::SocketAddr, ops::Deref, sync::Arc};

#[cfg(feature = "socket_io")]
use config::SocketIOConfig;

#[cfg(feature = "openapi")]
use crate::config::OpenApiConfig;

/// Routers collection
#[cfg(feature = "openapi")]
pub type Routers = Vec<aide::axum::ApiRouter>;
#[cfg(not(feature = "openapi"))]
pub type Routers = Vec<axum::Router>;

/// OpenAPI
#[cfg(feature = "openapi")]
type OpenApiTransformer = fn(TransformOpenApi) -> TransformOpenApi;

/// Web Configurator
pub trait WebConfigurator {
    /// add route to app registry
    fn add_router(&mut self, router: Router) -> &mut Self;

    /// Initialize OpenAPI Documents
    #[cfg(feature = "openapi")]
    fn openapi(&mut self, openapi: OpenApi) -> &mut Self;

    /// Defining OpenAPI Documents
    #[cfg(feature = "openapi")]
    fn api_docs(&mut self, api_docs: OpenApiTransformer) -> &mut Self;
}

impl WebConfigurator for AppBuilder {
    fn add_router(&mut self, router: Router) -> &mut Self {
        if let Some(routers) = self.get_component_ref::<Routers>() {
            unsafe {
                let raw_ptr = ComponentRef::into_raw(routers);
                let routers = &mut *(raw_ptr as *mut Routers);
                routers.push(router);
            }
            self
        } else {
            self.add_component(vec![router])
        }
    }

    /// Initialize OpenAPI Documents
    #[cfg(feature = "openapi")]
    fn openapi(&mut self, openapi: OpenApi) -> &mut Self {
        self.add_component(openapi)
    }

    #[cfg(feature = "openapi")]
    fn api_docs(&mut self, api_docs: OpenApiTransformer) -> &mut Self {
        self.add_component(api_docs)
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

        #[cfg(feature = "socket_io")]
        let socketio_config = app.get_config::<SocketIOConfig>().ok();

        // 1. collect router
        let routers = app.get_component_ref::<Routers>();
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
            router = crate::middleware::apply_middleware(router, middlewares);
        }

        #[cfg(feature = "socket_io")]
        if let Some(socketio_config) = socketio_config {
            use spring::tracing::info;
            
            info!("Configuring SocketIO with namespace: {}", socketio_config.default_namespace);
            
            let (layer, io) = socketioxide::SocketIo::builder()
                .build_layer();
            
            let ns_path = socketio_config.default_namespace.clone();
            let ns_path_for_closure = ns_path.clone();
            io.ns(ns_path, move |socket: socketioxide::extract::SocketRef| {
                use spring::tracing::info;
                
                info!(socket_id = ?socket.id, "New socket connected to namespace: {}", ns_path_for_closure);
                
                crate::handler::auto_socketio_setup(&socket);
            });
            
            router = router.layer(layer);
            app.add_component(io);
        }

        app.add_component(router);

        let server_conf = config.server;
        #[cfg(feature = "openapi")]
        {
            let openapi_conf = config.openapi;
            app.add_component(openapi_conf.clone());
        }

        app.add_scheduler(move |app: Arc<App>| {
            Box::new(Self::schedule(app, server_conf))
        });
    }
}

impl WebPlugin {
    async fn schedule(
        app: Arc<App>,
        config: ServerConfig,
    ) -> Result<String> {
        let router = app.get_expect_component::<Router>();

        // 2. bind tcp listener
        let addr = SocketAddr::from((config.binding, config.port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("bind tcp listener failed:{addr}"))?;
        tracing::info!("bind tcp listener: {addr}");

        // 3. openapi
        #[cfg(feature = "openapi")]
        let router = {
            let openapi_conf = app.get_expect_component::<OpenApiConfig>();
            finish_openapi(&app, router, openapi_conf)
        };

        // 4. axum server
        let router = router.layer(Extension(AppState { app }));

        tracing::info!("axum server started");
        if config.connect_info {
            // with client connect info
            let service = router.into_make_service_with_connect_info::<SocketAddr>();
            let server = axum::serve(listener, service);
            if config.graceful {
                server.with_graceful_shutdown(shutdown_signal()).await
            } else {
                server.await
            }
        } else {
            let service = router.into_make_service();
            let server = axum::serve(listener, service);
            if config.graceful {
                server.with_graceful_shutdown(shutdown_signal()).await
            } else {
                server.await
            }
        }
        .context("start axum server failed")?;

        Ok("axum schedule finished".to_string())
    }
}

#[cfg(feature = "openapi")]
pub fn enable_openapi() {
    aide::generate::on_error(|error| {
        tracing::error!("{error}");
    });
    aide::generate::extract_schemas(false);
}

#[cfg(feature = "openapi")]
fn finish_openapi(
    app: &App,
    router: aide::axum::ApiRouter,
    openapi_conf: OpenApiConfig,
) -> axum::Router {
    let router = router.nest_api_service(&openapi_conf.doc_prefix, docs_routes(&openapi_conf));

    let mut api = app.get_component::<OpenApi>().unwrap_or_else(|| OpenApi {
        info: openapi_conf.info,
        ..Default::default()
    });

    let router = if let Some(api_docs) = app.get_component::<OpenApiTransformer>() {
        router.finish_api_with(&mut api, api_docs)
    } else {
        router.finish_api(&mut api)
    };

    router.layer(Extension(Arc::new(api)))
}

#[cfg(feature = "openapi")]
pub fn docs_routes(OpenApiConfig { doc_prefix, info }: &OpenApiConfig) -> aide::axum::ApiRouter {
    let router = aide::axum::ApiRouter::new();
    let _openapi_path = &format!("{doc_prefix}/openapi.json");
    let _doc_title = &info.title;

    #[cfg(feature = "openapi-scalar")]
    let router = router.route(
        "/scalar",
        aide::scalar::Scalar::new(_openapi_path)
            .with_title(_doc_title)
            .axum_route(),
    );
    #[cfg(feature = "openapi-redoc")]
    let router = router.route(
        "/redoc",
        aide::redoc::Redoc::new(_openapi_path)
            .with_title(_doc_title)
            .axum_route(),
    );
    #[cfg(feature = "openapi-swagger")]
    let router = router.route(
        "/swagger",
        aide::swagger::Swagger::new(_openapi_path)
            .with_title(_doc_title)
            .axum_route(),
    );

    router.route("/openapi.json", axum::routing::get(serve_docs))
}

#[cfg(feature = "openapi")]
async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl aide::axum::IntoApiResponse {
    axum::response::IntoResponse::into_response(axum::Json(api.as_ref()))
}

#[cfg(feature = "openapi")]
pub fn default_transform<'a>(
    path_item: aide::transform::TransformPathItem<'a>,
) -> aide::transform::TransformPathItem<'a> {
    path_item
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, waiting for web server shutdown")
        },
        _ = terminate => {
            tracing::info!("Received kill signal, waiting for web server shutdown")
        },
    }
}
