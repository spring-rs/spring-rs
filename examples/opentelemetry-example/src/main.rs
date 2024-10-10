use anyhow::Context;
use spring::{auto_config, App};
use spring_opentelemetry::{
    OpenTelemetryPlugin, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_EXPORTER_OTLP_HEADERS,
};
use spring_sqlx::{
    sqlx::{self, Row},
    ConnectPool, SqlxPlugin,
};
use spring_web::{
    axum::response::IntoResponse,
    error::Result,
    extractor::{Component, Path},
    WebConfigurator, WebPlugin,
};
use spring_web::{get, route};

// Main function entry
#[auto_config(WebConfigurator)] // auto config web router
#[tokio::main]
async fn main() {
    std::env::set_var(OTEL_EXPORTER_OTLP_ENDPOINT, "localhost:5081");
    std::env::set_var(
        OTEL_EXPORTER_OTLP_HEADERS,
        "authorization=Basic cm9vdEBleGFtcGxlLmNvbTpaV1NYZnNDSlIwODJLdjhW,organization=default",
    );
    App::new()
        .add_plugin(SqlxPlugin) // Add plug-in
        .add_plugin(WebPlugin)
        .add_plugin(OpenTelemetryPlugin)
        .run()
        .await
}

// The get macro specifies the Http Method and request path.
// spring-rs also provides other standard http method macros such as post, delete, patch, etc.
#[get("/")]
async fn hello_world() -> impl IntoResponse {
    "hello world"
}

// You can also use the route macro to specify the Http Method and request path.
// Path extracts parameters from the HTTP request path
#[route("/hello/:name", method = "GET", method = "POST")]
async fn hello(Path(name): Path<String>) -> impl IntoResponse {
    format!("hello {name}")
}

// Component can extract the connection pool registered by the SqlxPlugin in AppState
#[get("/version")]
async fn sqlx_request_handler(Component(pool): Component<ConnectPool>) -> Result<String> {
    let version = sqlx::query("select version() as version")
        .fetch_one(&pool)
        .await
        .context("sqlx query failed")?
        .get("version");
    Ok(version)
}
