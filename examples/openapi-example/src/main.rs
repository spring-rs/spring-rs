use std::fmt::Display;

use schemars::JsonSchema;
use serde::Serialize;
use spring::{auto_config, App};
use spring_sqlx::SqlxPlugin;
use spring_web::axum::response::IntoResponse;
use spring_web::axum::Json;
use spring_web::get_api;
use spring_web::WebPlugin;
use spring_web::WebConfigurator;
use spring_web::ProblemDetails;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        // Add middleware to capture request URI for Problem Details
        .add_router(
            spring_web::Router::new()
                .route("/user-info/:id", spring_web::axum::routing::get(user_info_api))
                .layer(spring_web::axum::middleware::from_fn(
                    spring_web::problem_details::capture_request_uri_middleware
                ))
        )
        .run()
        .await;
}

#[derive(Debug, thiserror::Error, ProblemDetails)]
pub enum ApiErrors {
    // 基本用法：使用 about:blank 作为默认 problem_type
    #[status_code(400)]
    #[error("Invalid input provided")]
    BadRequest,

    // 部分自定义：自定义 title 和 detail (显式使用 title 属性)
    #[status_code(400)]
    #[problem_type("https://api.myapp.com/problems/email-validation")]
    #[title("Email Validation Failed")]
    #[detail("The provided email address is not valid")]
    #[error("Invalid email")]
    InvalidEmail,

    // 使用 error 属性作为 title (兼容性功能)
    #[status_code(422)]
    #[problem_type("https://api.myapp.com/problems/validation-failed")]
    #[detail("The request data failed validation checks")]
    #[error("Validation Failed")]  // 这个会自动用作 title
    ValidationFailed,

    // 完全自定义：所有字段都自定义
    #[status_code(401)]
    #[problem_type("https://api.myapp.com/problems/authentication-required")]
    #[title("Authentication Required")]
    #[detail("You must be authenticated to access this resource")]
    #[instance("/auth/login")]
    #[error("Authentication required")]
    AuthenticationRequired,

    #[status_code(403)]
    #[problem_type("https://api.myapp.com/problems/access-denied")]
    #[error("Access denied")]
    AuthorizationError,

    #[status_code(404)]
    #[problem_type("https://api.myapp.com/problems/resource-not-found")]
    #[error("Resource not found")]
    NotFoundError,

    #[status_code(500)]
    #[problem_type("https://api.myapp.com/problems/database-error")]
    #[error(transparent)]
    SqlxError(#[from] spring_sqlx::sqlx::Error),

    #[status_code(418)]
    #[problem_type("https://api.myapp.com/problems/teapot-error")]
    #[error("TeaPod error occurred: {0:?}")]
    TeaPod(CustomErrorSchema),

    // 自定义状态码：429 Too Many Requests (使用 error 属性作为 title)
    #[status_code(429)]
    #[problem_type("https://api.myapp.com/problems/rate-limit-exceeded")]
    #[detail("You have exceeded the maximum number of requests per minute")]
    #[error("Rate Limit Exceeded")]  // 这个会自动用作 title
    RateLimitExceeded,
}

// ToProblemDetails 现在是自动生成的！

impl IntoResponse for ApiErrors {
    fn into_response(self) -> spring_web::axum::response::Response {
        // ToProblemDetails 现在是自动生成的！
        self.to_problem_details().into_response()
    }
}

#[derive(Debug, JsonSchema)]
pub struct CustomErrorSchema {
    pub code: u16,
    pub message: String,
}

#[derive(Debug, Serialize, JsonSchema)]
struct UserInfo {
    user_id: i64,
    user_info: String,
}

impl UserInfo {
    fn new(user_id: i64, user_info: String) -> Self {
        Self { user_id, user_info }
    }
}

/// Get user information (with automatic URI capture)
/// 
/// This endpoint demonstrates automatic URI capture for Problem Details instance field.
/// The middleware automatically captures the request URI and includes it in error responses.
/// 
/// @tag User
/// @status_codes ApiErrors::BadRequest, ApiErrors::ValidationFailed, ApiErrors::AuthenticationRequired, ApiErrors::NotFoundError, ApiErrors::RateLimitExceeded
#[get_api("/user-info/{id}")]
async fn user_info_api(id: u32) -> Result<Json<UserInfo>, ApiErrors> {
    match id {
        0 => Err(ApiErrors::BadRequest),
        1 => Err(ApiErrors::InvalidEmail),
        2 => Err(ApiErrors::ValidationFailed),
        3 => Err(ApiErrors::AuthenticationRequired),
        4 => Err(ApiErrors::AuthorizationError),
        999 => Err(ApiErrors::NotFoundError), // Will automatically include "/user-info/999" as instance
        1000 => Err(ApiErrors::RateLimitExceeded),
        9999 => Err(ApiErrors::TeaPod(CustomErrorSchema { 
            code: 418, 
            message: "I'm a teapot".to_string() 
        })),
        _ => {
            let user_info = fetch_user_info(id as i64).await;
            if let Ok(info) = user_info {
                Ok(Json(info))
            } else {
                Err(ApiErrors::NotFoundError)
            }
        }
    }
}

async fn fetch_user_info(user_id: i64) -> Result<UserInfo, ApiErrors> {
    let is_database_connected = true;
    if !is_database_connected {
        return Err(ApiErrors::SqlxError(
            spring_sqlx::sqlx::Error::PoolTimedOut,
        ));
    }

    // Simulate fetching user info from a database or external service
    Ok(UserInfo::new(user_id, "Sample user info".to_string()))
}
