//! Vulcan Worker library.
//!
//! This crate provides the worker service that connects to the orchestrator,
//! requests work, executes scripts, and reports results.

pub mod client;
pub mod config;
pub mod error;
pub mod executor;
pub mod worker;
