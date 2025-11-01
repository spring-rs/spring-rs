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

use spring_web::HttpStatusCode;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SqlxPlugin)
        .add_plugin(WebPlugin)
        .run()
        .await;
}

#[derive(Debug, thiserror::Error, HttpStatusCode)]
pub enum CustomErrors {
    #[status_code(400)]
    #[error("A basic error occurred")]
    ABasicError,

    #[status_code(500)]
    #[error(transparent)]
    SqlxError(#[from] spring_sqlx::sqlx::Error),

    #[status_code(418)]
    #[error("TeaPod error occurred: {0:?}")]
    TeaPod(CustomErrorSchema),
}

#[derive(Debug, JsonSchema)]
pub struct CustomErrorSchema {
    pub code: u16,
    pub message: String,
}

impl IntoResponse for CustomErrors {
    fn into_response(self) -> spring_web::axum::response::Response {
        let body = format!("Error occurred: {}", self);
        (spring_web::axum::http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
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


/// Protected User info api
/// 
/// This endpoint fetches user information and demonstrates error handling with custom error types.
/// In this example, we simulate scenarios where a user may not be found or a database connection issue occurs.
/// 
/// @tag User
/// @status_codes CustomErrors::ABasicError, CustomErrors::SqlxError, CustomErrors::TeaPod
#[get_api("/user-info-api")]
async fn user_info_api() -> Result<Json<UserInfo>, CustomErrors> {
    let has_user = true;
    if !has_user {
        return Err(CustomErrors::ABasicError);
    }

    let user_info = fetch_user_info(42).await;

    if let Ok(info) = user_info {
        Ok(Json(info))
    } else {
        Err(CustomErrors::TeaPod(CustomErrorSchema { code: 5, message: "User info not found".to_string() }))
    }
}

async fn fetch_user_info(user_id: i64) -> Result<UserInfo, CustomErrors> {
    let is_database_connected = true;
    if !is_database_connected {
        return Err(CustomErrors::SqlxError(
            spring_sqlx::sqlx::Error::PoolTimedOut,
        ));
    }

    // Simulate fetching user info from a database or external service
    Ok(UserInfo::new(user_id, "Sample user info".to_string()))
}
