use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("No project loaded")]
    NoProject,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Index out of range: {0}")]
    IndexOutOfRange(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Pipeline error: {0}")]
    Pipeline(#[from] cm_pipeline::PipelineError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            ServerError::NoProject => (StatusCode::BAD_REQUEST, "no_project"),
            ServerError::SessionNotFound => (StatusCode::NOT_FOUND, "session_not_found"),
            ServerError::IndexOutOfRange(_) => (StatusCode::BAD_REQUEST, "index_out_of_range"),
            ServerError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            ServerError::BadRequest(_) => (StatusCode::BAD_REQUEST, "bad_request"),
            ServerError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal"),
            ServerError::Pipeline(_) => (StatusCode::UNPROCESSABLE_ENTITY, "pipeline_error"),
            ServerError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "io_error"),
        };

        let body = json!({
            "error": self.to_string(),
            "code": code,
        });

        (status, Json(body)).into_response()
    }
}

/// Helper to convert Mutex poison errors.
pub fn lock_err<T>(e: std::sync::PoisonError<T>) -> ServerError {
    ServerError::Internal(format!("Lock poisoned: {e}"))
}
