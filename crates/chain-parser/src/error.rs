//! Error types for the chain parser.

use thiserror::Error;

/// Errors that can occur during chain parsing.
#[derive(Debug, Error)]
pub enum ParseError {
    /// KDL syntax error.
    #[error("invalid KDL syntax: {0}")]
    InvalidSyntax(String),

    /// Required field is missing.
    #[error("missing required field: {field} in {context}")]
    MissingRequired {
        /// The missing field name.
        field: &'static str,
        /// Where the field was expected.
        context: String,
    },

    /// Invalid URL format.
    #[error("invalid URL: {0}")]
    InvalidUrl(String),

    /// Failed to fetch an import URL.
    #[error("failed to fetch import from {url}: {reason}")]
    FetchFailed {
        /// The URL that failed.
        url: String,
        /// The reason for failure.
        reason: String,
    },

    /// Circular import detected.
    #[error("circular import detected: {0}")]
    CircularImport(String),

    /// Mutual exclusion violation (both `run` and `from` specified).
    #[error("fragment cannot have both 'run' and 'from'")]
    MutualExclusion,

    /// Fragment has neither `run` nor `from`.
    #[error("fragment must have either 'run' or 'from'")]
    NoContent,

    /// No machine specified at chain or fragment level.
    #[error("no machine specified for fragment and no default machine in chain")]
    NoMachine,

    /// Unknown node type encountered.
    #[error("unknown node type: {0}")]
    UnknownNode(String),

    /// Invalid node in imported file.
    #[error("imported files can only contain fragment/parallel nodes, found: {0}")]
    InvalidImportNode(String),

    /// Invalid version.
    #[error("unsupported version: {0}")]
    UnsupportedVersion(String),

    /// Invalid trigger type.
    #[error("invalid trigger type: {0}")]
    InvalidTrigger(String),
}

/// Result type for parser operations.
pub type Result<T> = std::result::Result<T, ParseError>;
