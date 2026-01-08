use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Establishes a connection to the `PostgreSQL` database.
///
/// # Panics
///
/// Panics if `DATABASE_URL` environment variable is not set or if the connection fails.
pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

/// Runs all pending database migrations.
///
/// # Panics
///
/// Panics if migrations fail to run.
pub fn run_migrations(connection: &mut PgConnection) {
    connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");
}
