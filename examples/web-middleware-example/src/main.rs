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
use tower_http::cors::CorsLayer;
use spring_web::nest;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(WebPlugin)
        .add_plugin(SqlxPlugin)
        .run()
        .await
}

/// Example #1:

/// Example of using `middlewares` macro to apply middleware to all routes in a module.
/// This module includes a problem detail middleware that handles errors and logs them to the database.
/// It also includes a timeout layer to limit request processing time.
/// The `hello_world` route returns a simple greeting, while the `sql_version` route
/// queries the database for its version. The `error_request` route demonstrates error handling.
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

/// Example #2:
/// This example demonstrates how to use the `middlewares` macro to apply multiple middleware layers to a module.
/// It includes a logging middleware, an authentication middleware, and a timeout layer.
/// The `protected` route is protected by the authentication middleware, which checks for an `Authorization` header.
/// If the header is missing, it returns a 401 Unauthorized response.

#[middlewares(
    middleware::from_fn(logging_middleware),
    middleware::from_fn(auth_middleware),
    TimeoutLayer::new(Duration::from_secs(10)),
    CorsLayer::permissive()
)]
mod protected_routes {
    use super::*;

    #[get("/protected")]
    async fn protected() -> impl IntoResponse {
        "Protected endpoint!"
    }
}

async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    println!("🔍 [LOGGING] {} {}", request.method(), request.uri().path());
    let response = next.run(request).await;
    println!("✅ [LOGGING] Response completed");
    response
}

async fn auth_middleware(
    request: Request,
    next: Next,
) -> Response {
    println!("🔐 [AUTH] Checking authentication for: {}", request.uri().path());
    
    if request.headers().get("Authorization").is_none() {
        return Response::builder()
            .status(401)
            .body("Unauthorized".into())
            .unwrap();
    }

    next.run(request).await
}

/// Example #3:

#[middlewares(
    middleware::from_fn(logging_middleware),
    middleware::from_fn(auth_middleware),
    TimeoutLayer::new(Duration::from_secs(10)),
    CorsLayer::permissive()
)]
#[nest("/api")]
mod api {

    use super::*;

    #[get("/hello")]
    pub async fn hello() -> impl IntoResponse {
        "Hello, world!"
    }
}

/// Example #4:
/// This route demonstrates a simple goodbye endpoint without any middleware.
/// It returns a static string "goodbye world" when accessed.

#[get("/goodbye")]
async fn goodbye_world() -> impl IntoResponse {
    "goodbye world"
}

