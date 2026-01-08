use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

/// Establishes a connection to the `PostgreSQL` database.
///
/// # Panics
///
/// Panics if `DATABASE_URL` environment variable is not set or if the connection fails.
#[must_use]
pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

/// Runs all pending database migrations.
///
/// This function is only available when the `migrations` feature is enabled.
/// It is designed to be called by the API service which owns the migrations.
///
/// # Panics
///
/// Panics if migrations fail to run.
#[cfg(feature = "migrations")]
pub fn run_migrations(connection: &mut PgConnection) {
    use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

    pub const MIGRATIONS: EmbeddedMigrations =
        diesel_migrations::embed_migrations!("../../migrations");

    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
}

#[cfg(not(feature = "migrations"))]
/// Placeholder for `run_migrations` when migrations feature is disabled.
///
/// # Panics
///
/// Always panics when called without the migrations feature enabled.
pub fn run_migrations(_connection: &mut PgConnection) {
    panic!("Migrations feature is not enabled. Only the API service should run migrations.");
}
