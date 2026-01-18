//! Error types for the worker orchestrator.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;

/// Errors that can occur in the worker orchestrator.
#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Database error.
    #[error("Database error: {0}")]
    Database(#[from] vulcan_core::repositories::RepositoryError),

    /// Connection pool error.
    #[error("Connection pool error: {0}")]
    Pool(#[from] diesel::r2d2::PoolError),

    /// Worker not found.
    #[error("Worker not found: {0}")]
    WorkerNotFound(uuid::Uuid),

    /// Fragment not found.
    #[error("Fragment not found: {0}")]
    FragmentNotFound(uuid::Uuid),

    /// Chain not found.
    #[error("Chain not found: {0}")]
    ChainNotFound(uuid::Uuid),

    /// No work available.
    #[error("No work available")]
    NoWorkAvailable,

    /// Invalid request.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

/// Error response body.
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for OrchestratorError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Self::Database(_) | Self::Pool(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string())
            }
            Self::WorkerNotFound(_) | Self::FragmentNotFound(_) | Self::ChainNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            Self::NoWorkAvailable => (StatusCode::NO_CONTENT, self.to_string()),
            Self::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

/// Result type alias for orchestrator operations.
pub type Result<T> = std::result::Result<T, OrchestratorError>;
