//! Vulcan Chain Parser API Library.
//!
//! This module exposes the API components for use in tests and the main binary.

use std::sync::{Arc, Mutex};

use axum::routing::{get, post};
use axum::Router;
use diesel::pg::PgConnection;

pub mod error;
pub mod handlers;

pub use handlers::AppState;

/// Build the API router with the given application state.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/parse", post(handlers::parse_workflow))
        .with_state(state)
}

/// Create application state from a database connection.
pub fn create_app_state(conn: PgConnection) -> Arc<AppState> {
    Arc::new(AppState {
        db: Mutex::new(conn),
    })
}
