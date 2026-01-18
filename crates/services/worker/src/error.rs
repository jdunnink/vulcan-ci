//! Error types for the worker service.

use thiserror::Error;

/// Result type for worker operations.
pub type Result<T> = std::result::Result<T, WorkerError>;

/// Errors that can occur in the worker service.
#[derive(Debug, Error)]
pub enum WorkerError {
    /// Missing required environment variable.
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    /// Invalid configuration value.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// HTTP client error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Worker not registered.
    #[error("Worker not registered with orchestrator")]
    NotRegistered,

    /// Script execution error.
    #[error("Script execution error: {0}")]
    ScriptExecution(String),

    /// Script execution timed out.
    #[error("Script execution timed out after {0} seconds")]
    ScriptTimeout(u64),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Orchestrator returned an error.
    #[error("Orchestrator error: {0}")]
    Orchestrator(String),
}
