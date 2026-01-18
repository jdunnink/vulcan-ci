//! API module for the worker orchestrator.

pub mod dto;
pub mod handlers;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

/// Create the API router with all endpoints.
pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/workers/register", post(handlers::register_worker))
        .route("/workers/heartbeat", post(handlers::heartbeat))
        .route("/work/request", post(handlers::request_work))
        .route("/work/result", post(handlers::report_result))
        .with_state(state)
}
