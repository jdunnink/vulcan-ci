//! Vulcan Chain Parser API Service.
//!
//! HTTP service for parsing workflow files and storing them in PostgreSQL.

use std::env;
use std::net::SocketAddr;

use tracing_subscriber::EnvFilter;

use vulcan_chain_parser_api::{build_router, create_app_state};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Verify DATABASE_URL is set
    if env::var("DATABASE_URL").is_err() {
        tracing::error!("DATABASE_URL environment variable must be set");
        std::process::exit(1);
    }

    // Run migrations (uses DATABASE_URL from env)
    let mut migration_conn = vulcan_core::establish_connection();
    vulcan_core::run_migrations(&mut migration_conn);
    drop(migration_conn);
    tracing::info!("Database migrations complete");

    // Establish connection for the service
    let conn = vulcan_core::establish_connection();
    tracing::info!("Connected to database");

    // Create application state and router
    let state = create_app_state(conn);
    let app = build_router(state);

    // Start server
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3001);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
