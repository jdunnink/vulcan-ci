//! Error types for the worker-controller service.

use thiserror::Error;

/// Errors that can occur in the worker-controller.
#[derive(Error, Debug)]
pub enum ControllerError {
    /// HTTP request error.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// Kubernetes API error.
    #[error("Kubernetes API error: {0}")]
    Kube(#[from] kube::Error),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Deployment not found.
    #[error("Deployment {name} not found in namespace {namespace}")]
    DeploymentNotFound {
        /// Deployment name.
        name: String,
        /// Deployment namespace.
        namespace: String,
    },
}

/// Result type for controller operations.
pub type Result<T> = std::result::Result<T, ControllerError>;
