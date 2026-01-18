//! Vulcan Worker Orchestrator Library.
//!
//! This crate provides the core functionality for the worker orchestrator service,
//! including API handlers, scheduling logic, and health monitoring.

pub mod api;
pub mod config;
pub mod error;
pub mod orchestrator;
pub mod state;

pub use config::Config;
pub use error::{OrchestratorError, Result};
pub use state::AppState;
