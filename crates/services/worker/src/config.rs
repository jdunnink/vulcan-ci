//! Configuration for the worker service.

use std::env;
use std::time::Duration;

use uuid::Uuid;

use crate::error::{Result, WorkerError};

/// Sandbox configuration for script execution.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    /// Whether sandbox is enabled.
    pub enabled: bool,
    /// Memory limit for sandboxed processes (e.g., "512M").
    pub memory_limit: String,
    /// Whether to allow network access in sandbox.
    pub network: bool,
    /// Scratch directory for script execution.
    pub scratch_dir: String,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            memory_limit: "512M".to_string(),
            network: false,
            scratch_dir: "/scratch".to_string(),
        }
    }
}

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
    /// Sandbox configuration.
    pub sandbox: SandboxConfig,
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

        let sandbox = SandboxConfig {
            enabled: env::var("SANDBOX_ENABLED")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(true),
            memory_limit: env::var("SANDBOX_MEMORY_LIMIT")
                .unwrap_or_else(|_| "512M".to_string()),
            network: env::var("SANDBOX_NETWORK")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(false),
            scratch_dir: env::var("SANDBOX_SCRATCH_DIR")
                .unwrap_or_else(|_| "/scratch".to_string()),
        };

        Ok(Self {
            orchestrator_url,
            tenant_id,
            worker_group,
            heartbeat_interval,
            poll_interval,
            request_timeout,
            script_timeout,
            sandbox,
        })
    }
}
