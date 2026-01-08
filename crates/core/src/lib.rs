//! Vulcan Core - Shared data models and repositories.
//!
//! This crate provides the core data structures, database schema,
//! and repository implementations used across all Vulcan services.

/// Database connection and migration utilities.
pub mod db;
/// Data models for domain entities.
pub mod models;
/// Repository pattern implementations.
pub mod repositories;
/// Auto-generated Diesel schema definitions.
#[allow(missing_docs, clippy::wildcard_imports)]
pub mod schema;

pub use db::{establish_connection, run_migrations};
pub use models::{
    chain::{Chain, ChainStatus, NewChain},
    fragment::{Fragment, FragmentStatus, NewFragment},
    worker::{NewWorker, Worker, WorkerStatus},
};
pub use repositories::{
    ChainRepository, FragmentRepository, PgChainRepository, PgFragmentRepository,
    PgWorkerRepository, RepositoryError, WorkerRepository,
};
