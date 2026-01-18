//! Vulcan API Service.
//!
//! Main HTTP API for managing workflows, chains, and workers.
//! This service is responsible for applying database migrations.

use vulcan_core::{establish_connection, run_migrations};

fn main() {
    println!("Hello from Vulcan API!");

    // Load environment variables
    dotenvy::dotenv().ok();

    // Establish database connection and run migrations
    let mut conn = establish_connection();
    run_migrations(&mut conn);

    println!("Database migrations applied successfully.");
}
