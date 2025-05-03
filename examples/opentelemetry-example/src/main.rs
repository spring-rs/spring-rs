use anyhow::Context;
use spring::{
    tracing::{info, info_span, Instrument, Level},
    App,
};
use spring_opentelemetry::{
    middlewares, KeyValue, OpenTelemetryPlugin, ResourceConfigurator, SERVICE_NAME, SERVICE_VERSION,
};
use spring_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin,
};
use spring_web::{
    axum::response::IntoResponse,
    error::Result,
    extractor::{Component, Path},
    middleware::trace::{
        DefaultMakeSpan, DefaultOnEos, DefaultOnRequest, DefaultOnResponse, TraceLayer,
    },
    Router, WebConfigurator, WebPlugin,
};
use spring_web::{get, route};

// Main function entry
#[tokio::main]
async fn main() {
    App::new()
        .opentelemetry_attrs([
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
        ])
        .add_router(router())
        .add_plugin(SqlxPlugin) // Add plug-in
        .add_plugin(WebPlugin)
        .add_plugin(OpenTelemetryPlugin)
        .run()
        .await
}

fn router() -> Router {
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::default())
        .on_request(DefaultOnRequest::default())
        .on_response(DefaultOnResponse::default())
        .on_eos(DefaultOnEos::default());
    let http_tracing_layer =
        middlewares::tracing::HttpLayer::server(Level::INFO).export_trace_id(true);
    // Note: http_tracing_layer must be added after trace_layer, because axum defaults to adding it first and executing it later.
    spring_web::handler::auto_router()
        .layer(trace_layer)
        .layer(http_tracing_layer)
}

// The get macro specifies the Http Method and request path.
// spring-rs also provides other standard http method macros such as post, delete, patch, etc.
#[get("/")]
async fn hello_world() -> impl IntoResponse {
    info!("hello world called");
    "hello world"
}

// You can also use the route macro to specify the Http Method and request path.
// Path extracts parameters from the HTTP request path
#[route("/hello/{name}", method = "GET", method = "POST")]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    info!("hello {name} called");
    format!("hello {name}")
}

// Component can extract the connection pool registered by the SqlxPlugin in AppState
#[get("/version")]
async fn sqlx_request_handler(Component(pool): Component<ConnectPool>) -> Result<String> {
    info!("query sqlx version called");
    let version = sqlx::query("select version() as version")
        .fetch_one(&pool)
        .instrument(info_span!("sqlx-query"))
        .await
        .context("sqlx query failed")?
        .get("version");
    Ok(version)
}
