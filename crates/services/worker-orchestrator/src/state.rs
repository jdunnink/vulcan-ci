//! Application state for the worker orchestrator.

use std::sync::Arc;

use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

use crate::config::Config;

/// Type alias for the database connection pool.
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

/// Application state shared across all request handlers.
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool.
    pub pool: DbPool,
    /// Service configuration.
    pub config: Arc<Config>,
}

impl AppState {
    /// Create a new application state with the given configuration.
    ///
    /// # Panics
    /// Panics if the database connection pool cannot be created.
    pub fn new(config: Config) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
        let pool = r2d2::Pool::builder()
            .max_size(10)
            .build(manager)
            .expect("Failed to create database connection pool");

        Self {
            pool,
            config: Arc::new(config),
        }
    }

    /// Get a connection from the pool.
    ///
    /// # Errors
    /// Returns an error if a connection cannot be acquired from the pool.
    pub fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, r2d2::PoolError> {
        self.pool.get()
    }
}
