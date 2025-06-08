use anyhow::Context;
use spring::{auto_config, App};
use spring_sqlx::sqlx::Row;
use spring_sqlx::{sqlx, ConnectPool, SqlxPlugin};
use spring_web::error::KnownWebError;
use spring_web::{middlewares, WebConfigurator};
use spring_web::{
    axum::{
        body,
        middleware::{self, Next},
        response::{IntoResponse, Response},
    },
    error::Result,
    extractor::Component,
    extractor::Request,
    WebPlugin,
};
use std::time::Duration;
use tower_http::timeout::TimeoutLayer;
use spring_web::get;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .add_plugin(SqlxPlugin)
        .run()
        .await
}

#[middlewares(
    middleware::from_fn(problem_middleware),
    TimeoutLayer::new(Duration::from_secs(10))
)]
mod routes {
    use super::*;

    #[get("/")]
    async fn hello_world() -> impl IntoResponse {
        "hello world"
    }

    #[get("/version")]
    async fn sql_version(Component(pool): Component<ConnectPool>) -> Result<String> {
        let version = sqlx::query("select version() as version")
            .fetch_one(&pool)
            .await
            .context("sqlx query failed")?
            .get("version");
        Ok(version)
    }

    #[get("/error")]
    async fn error_request() -> Result<String> {
        Err(KnownWebError::bad_request("request error"))?
    }

}

#[get("/goodbye")]
async fn goodbye_world() -> impl IntoResponse {
    "goodbye world"
}

/// ProblemDetail: https://www.rfc-editor.org/rfc/rfc7807
async fn problem_middleware(
    Component(db): Component<ConnectPool>,
    request: Request,
    next: Next,
) -> Response {
    let uri = request.uri().path().to_string();
    let response = next.run(request).await;
    let status = response.status();
    if status.is_client_error() || status.is_server_error() {
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("server body read failed");
        let msg = String::from_utf8(bytes.to_vec()).expect("read body to string failed");

        // error log into db
        let _ = sqlx::query("insert into error_log (msg, created_at) values ($1, now())")
            .bind(&msg)
            .execute(&db)
            .await;

        problemdetails::new(status)
            .with_instance(uri)
            .with_title(status.canonical_reason().unwrap_or("error"))
            .with_detail(msg)
            .into_response()
    } else {
        response
    }
}
