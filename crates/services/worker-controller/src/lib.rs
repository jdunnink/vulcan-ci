//! Vulcan Worker Controller
//!
//! A Kubernetes-aware service that scales worker Deployments based on
//! job queue depth from the orchestrator.
//!
//! # Architecture
//!
//! The worker-controller runs on client Kubernetes infrastructure and:
//! 1. Polls the orchestrator for queue metrics (pending/running fragments)
//! 2. Calculates desired replica count based on pending work
//! 3. Scales the worker Deployment up or down accordingly
//!
//! # Configuration
//!
//! The controller is configured via environment variables:
//!
//! ## Required
//! - `ORCHESTRATOR_URL`: URL of the orchestrator service
//! - `TENANT_ID`: UUID of the tenant
//! - `MACHINE_GROUP`: Machine group to manage
//! - `DEPLOYMENT_NAME`: Kubernetes deployment name
//! - `DEPLOYMENT_NAMESPACE`: Kubernetes deployment namespace
//!
//! ## Scaling parameters (with defaults)
//! - `MIN_REPLICAS`: Minimum replicas (default: 0)
//! - `MAX_REPLICAS`: Maximum replicas (default: 10)
//! - `TARGET_PENDING_PER_WORKER`: Target pending per worker (default: 1.0)
//! - `SCALE_DOWN_DELAY_SECONDS`: Scale down delay (default: 300)
//! - `POLL_INTERVAL_SECONDS`: Poll interval (default: 30)

pub mod client;
pub mod config;
pub mod controller;
pub mod error;
pub mod kubernetes;
pub mod scaler;

pub use config::Config;
pub use controller::Controller;
pub use error::{ControllerError, Result};
