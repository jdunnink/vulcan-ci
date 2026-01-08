use std::fmt;

/// Error type for repository operations.
#[derive(Debug)]
pub enum RepositoryError {
    /// Record was not found.
    NotFound,
    /// Database error occurred.
    DatabaseError(diesel::result::Error),
    /// A conflict occurred (e.g., duplicate key).
    Conflict(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => write!(f, "Record not found"),
            Self::DatabaseError(e) => write!(f, "Database error: {e}"),
            Self::Conflict(msg) => write!(f, "Conflict: {msg}"),
        }
    }
}

impl std::error::Error for RepositoryError {}

impl From<diesel::result::Error> for RepositoryError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::NotFound => Self::NotFound,
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                info,
            ) => Self::Conflict(info.message().to_string()),
            _ => Self::DatabaseError(error),
        }
    }
}

/// Result type alias for repository operations.
pub type Result<T> = std::result::Result<T, RepositoryError>;
