//! Request/Response models for Sa-Token example

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct UserInfo {
    pub user_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct MessageResponse {
    pub message: String,
}

#[derive(Serialize)]
pub struct TokenInfo {
    pub token: String,
    pub login_id: String,
    pub is_login: bool,
}