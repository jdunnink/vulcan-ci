//! Configuration for the worker service.

use std::env;
use std::time::Duration;

use uuid::Uuid;

use crate::error::{Result, WorkerError};

/// Worker configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Orchestrator endpoint URL.
    pub orchestrator_url: String,
    /// Tenant ID this worker belongs to.
    pub tenant_id: Uuid,
    /// Machine group for this worker (optional).
    pub worker_group: Option<String>,
    /// Heartbeat interval.
    pub heartbeat_interval: Duration,
    /// Work polling interval.
    pub poll_interval: Duration,
    /// HTTP request timeout.
    pub request_timeout: Duration,
    /// Script execution timeout.
    pub script_timeout: Duration,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing or invalid.
    pub fn from_env() -> Result<Self> {
        let orchestrator_url = env::var("ORCHESTRATOR_URL")
            .map_err(|_| WorkerError::MissingEnvVar("ORCHESTRATOR_URL".to_string()))?;

        let tenant_id_str = env::var("TENANT_ID")
            .map_err(|_| WorkerError::MissingEnvVar("TENANT_ID".to_string()))?;
        let tenant_id = Uuid::parse_str(&tenant_id_str)
            .map_err(|e| WorkerError::InvalidConfig(format!("Invalid TENANT_ID: {e}")))?;

        let worker_group = env::var("WORKER_GROUP").ok();

        let heartbeat_interval = Duration::from_secs(
            env::var("HEARTBEAT_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
        );

        let poll_interval = Duration::from_secs(
            env::var("POLL_INTERVAL_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
        );

        let request_timeout = Duration::from_secs(
            env::var("REQUEST_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
        );

        let script_timeout = Duration::from_secs(
            env::var("SCRIPT_TIMEOUT_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300),
        );

        Ok(Self {
            orchestrator_url,
            tenant_id,
            worker_group,
            heartbeat_interval,
            poll_interval,
            request_timeout,
            script_timeout,
        })
    }
}
