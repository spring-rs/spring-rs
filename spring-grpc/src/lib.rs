pub mod config;

pub use tonic;

use anyhow::Context;
use config::GrpcConfig;
use http::Request;
use spring::{
    app::AppBuilder,
    config::ConfigRegistry,
    error::Result,
    plugin::{component::ComponentRef, ComponentRegistry, MutableComponentRegistry, Plugin},
};
use std::{convert::Infallible, net::SocketAddr};
use tonic::{
    async_trait,
    body::Body,
    server::NamedService,
    service::{Routes, RoutesBuilder},
    transport::Server,
};
use tower::Service;

/// Grpc Configurator
pub trait GrpcConfigurator {
    /// add grpc service to app registry
    fn add_service<S>(&mut self, service: S) -> &mut Self
    where
        S: Service<Request<Body>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Response: axum::response::IntoResponse,
        S::Future: Send + 'static;
}

impl GrpcConfigurator for AppBuilder {
    fn add_service<S>(&mut self, svc: S) -> &mut Self
    where
        S: Service<Request<Body>, Error = Infallible>
            + NamedService
            + Clone
            + Send
            + Sync
            + 'static,
        S::Response: axum::response::IntoResponse,
        S::Future: Send + 'static,
    {
        if let Some(routes) = self.get_component_ref::<RoutesBuilder>() {
            unsafe {
                let raw_ptr = ComponentRef::into_raw(routes);
                let routes = &mut *(raw_ptr as *mut RoutesBuilder);
                routes.add_service(svc);
            }
            self
        } else {
            let mut route_builder = Routes::builder();
            route_builder.add_service(svc);
            self.add_component(route_builder)
        }
    }
}

/// Grpc Plugin Definition
pub struct GrpcPlugin;

#[async_trait]
impl Plugin for GrpcPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app
            .get_config::<GrpcConfig>()
            .expect("grpc plugin config load failed");

        let routes_builder = app.get_component::<RoutesBuilder>();

        if let Some(routes_builder) = routes_builder {
            let routes = routes_builder.routes();
            app.add_scheduler(move |_app| Box::new(Self::schedule(config, routes)));
        } else {
            tracing::warn!(
                "The grpc plugin does not register any routes, so no scheduling is performed"
            );
        }
    }
}

impl GrpcPlugin {
    async fn schedule(config: GrpcConfig, routes: Routes) -> Result<String> {
        let mut server = Server::builder()
            .accept_http1(config.accept_http1)
            .http2_adaptive_window(config.http2_adaptive_window)
            .http2_keepalive_interval(config.http2_keepalive_interval)
            .http2_keepalive_timeout(config.http2_keepalive_timeout)
            .http2_max_header_list_size(config.http2_max_header_list_size)
            .http2_max_pending_accept_reset_streams(config.http2_max_pending_accept_reset_streams)
            .initial_connection_window_size(config.initial_connection_window_size)
            .initial_stream_window_size(config.initial_stream_window_size)
            .max_concurrent_streams(config.max_concurrent_streams)
            .max_frame_size(config.max_frame_size)
            .tcp_keepalive(config.tcp_keepalive)
            .tcp_nodelay(config.tcp_nodelay);

        if let Some(max_connection_age) = config.max_connection_age {
            server = server.max_connection_age(max_connection_age);
        }
        if let Some(timeout) = config.timeout {
            server = server.timeout(timeout);
        }
        if let Some(concurrency_limit_per_connection) = config.concurrency_limit_per_connection {
            server = server.concurrency_limit_per_connection(concurrency_limit_per_connection);
        }

        server = Self::apply_middleware(server);

        let addr = SocketAddr::new(config.binding, config.port);
        tracing::info!("tonic grpc service bind tcp listener: {}", addr);

        let router = server.add_routes(routes);
        if config.graceful {
            router
                .serve_with_shutdown(addr, shutdown_signal())
                .await
                .with_context(|| format!("bind tcp listener failed:{}", addr))?;
        } else {
            router
                .serve(addr)
                .await
                .with_context(|| format!("bind tcp listener failed:{}", addr))?;
        }
        Ok("tonic server schedule finished".to_string())
    }

    fn apply_middleware(_server: Server) -> Server {
        todo!()
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
            tracing::info!("Received Ctrl+C signal, waiting for tonic grpc server shutdown")
        },
        _ = terminate => {
            tracing::info!("Received kill signal, waiting for tonic grpc server shutdown")
        },
    }
}
