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

pub use axum;
pub use spring::async_trait;
/////////////////web-macros/////////////////////
/// To use these Procedural Macros, you need to add `spring-web` dependency
pub use spring_macros::delete;
pub use spring_macros::get;
pub use spring_macros::head;
pub use spring_macros::middlewares;
pub use spring_macros::nest;
pub use spring_macros::options;
pub use spring_macros::patch;
pub use spring_macros::post;
pub use spring_macros::put;
pub use spring_macros::route;
pub use spring_macros::routes;
pub use spring_macros::trace;

/// axum::routing::MethodFilter re-export
pub use axum::routing::MethodFilter;
/// MethodRouter with AppState
pub use axum::routing::MethodRouter;
/// Router with AppState
pub use axum::Router;

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

/// Routers collection
pub type Routers = Vec<Router>;

/// Web Configurator
pub trait WebConfigurator {
    /// add route to app registry
    fn add_router(&mut self, router: Router) -> &mut Self;
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

        let server_conf = config.server;

        app.add_scheduler(move |app: Arc<App>| Box::new(Self::schedule(router, app, server_conf)));
    }
}

impl WebPlugin {
    async fn schedule(router: Router, app: Arc<App>, config: ServerConfig) -> Result<String> {
        // 2. bind tcp listener
        let addr = SocketAddr::from((config.binding, config.port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .with_context(|| format!("bind tcp listener failed:{addr}"))?;
        tracing::info!("bind tcp listener: {addr}");

        // 3. axum server
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
