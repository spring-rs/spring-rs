use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, WebError>;

#[derive(Error, Debug)]
#[error(transparent)]
pub struct WebError(#[from] anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for WebError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}
