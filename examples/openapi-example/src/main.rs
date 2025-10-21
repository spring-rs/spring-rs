use schemars::JsonSchema;
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

#[derive(thiserror::Error, Debug, HttpStatusCode)]
pub enum Errors {
    #[status_code(401)]
    #[error("A error")]
    A,
    #[status_code(403)]
    #[error("B error")]
    B,
    #[status_code(500)]
    #[error("C error")]
    C,

    #[status_code(500)]
    #[error(transparent)]
    SqlxError(#[from] spring_sqlx::sqlx::Error),

    #[status_code(418)]
    #[error("Otra cosa")]
    TeaPod(Test),
}

#[derive(Debug, JsonSchema)]
pub struct Test {
    pub code: u16,
    pub message: String,
}

impl IntoResponse for Errors {
    fn into_response(self) -> spring_web::axum::response::Response {
        let body = format!("Error occurred: {}", self);
        (spring_web::axum::http::StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

/// Always return error  
/// 
/// This endpoint is annotated with status_codes for Errors::B and Errors::C
/// @tag error
/// @status_codes Errors::B, Errors::C, Errors::SqlxError, Errors::TeaPod
#[get_api("/error")]
async fn error() -> Result<Json<String>, Errors> {

    Err(Errors::B)
}


/// Always return error  
/// 
/// This **endpoint** is annotated with status_codes for Errors::A
/// @tag error
/// @status_codes Errors::A
#[get_api("/other_error")]
async fn other_error() -> Result<Json<String>, Errors> {

    Err(Errors::A)
}
