//! Repository pattern implementations for data access.
//!
//! This module provides traits and implementations for accessing
//! domain entities in a storage-agnostic way.

// Repository methods return Result types with self-explanatory error conditions
#![allow(clippy::missing_errors_doc)]

mod chain;
mod error;
mod fragment;
mod worker;

pub use chain::{ChainRepository, PgChainRepository};
pub use error::RepositoryError;
pub use fragment::{FragmentRepository, PgFragmentRepository};
pub use worker::{PgWorkerRepository, WorkerRepository};

/// Re-export the Result type for convenience.
pub type Result<T> = std::result::Result<T, RepositoryError>;
