//! Error types for the chain parser API.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// API error response body.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error message.
    pub error: String,
    /// Error code for programmatic handling.
    pub code: String,
}

/// API errors that can occur during request handling.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Workflow parsing failed.
    #[error("parse error: {0}")]
    ParseError(#[from] vulcan_chain_parser::ParseError),

    /// Database operation failed.
    #[error("database error: {0}")]
    DatabaseError(#[from] vulcan_core::RepositoryError),

    /// Invalid request body.
    #[error("invalid request: {0}")]
    InvalidRequest(String),

    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::ParseError(_) => (StatusCode::BAD_REQUEST, "PARSE_ERROR"),
            Self::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            Self::InvalidRequest(_) => (StatusCode::BAD_REQUEST, "INVALID_REQUEST"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        let body = ErrorResponse {
            error: self.to_string(),
            code: code.to_string(),
        };

        (status, Json(body)).into_response()
    }
}
