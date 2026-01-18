//! Configuration for the worker-controller service.

use std::env;
use uuid::Uuid;

/// Configuration for the worker-controller.
#[derive(Debug, Clone)]
pub struct Config {
    /// URL of the orchestrator service.
    pub orchestrator_url: String,
    /// Tenant ID for this controller (used for metrics filtering).
    pub tenant_id: Uuid,
    /// Machine group to manage.
    pub machine_group: String,
    /// Kubernetes deployment name.
    pub deployment_name: String,
    /// Kubernetes deployment namespace.
    pub deployment_namespace: String,
    /// Scaling configuration.
    pub scaling: ScalingConfig,
}

/// Scaling configuration for the controller.
#[derive(Debug, Clone)]
pub struct ScalingConfig {
    /// Minimum number of replicas.
    pub min_replicas: i32,
    /// Maximum number of replicas.
    pub max_replicas: i32,
    /// Target pending fragments per worker.
    pub target_pending_per_worker: f64,
    /// Delay in seconds before scaling down.
    pub scale_down_delay_seconds: i64,
    /// Interval in seconds between scaling checks.
    pub poll_interval_seconds: i64,
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            min_replicas: 0,
            max_replicas: 10,
            target_pending_per_worker: 1.0,
            scale_down_delay_seconds: 300,
            poll_interval_seconds: 30,
        }
    }
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Required environment variables
    /// - `ORCHESTRATOR_URL`: URL of the orchestrator service
    /// - `TENANT_ID`: UUID of the tenant
    /// - `MACHINE_GROUP`: Machine group to manage
    /// - `DEPLOYMENT_NAME`: Kubernetes deployment name
    /// - `DEPLOYMENT_NAMESPACE`: Kubernetes deployment namespace
    ///
    /// # Optional environment variables (with defaults)
    /// - `MIN_REPLICAS`: Minimum replicas (default: 0)
    /// - `MAX_REPLICAS`: Maximum replicas (default: 10)
    /// - `TARGET_PENDING_PER_WORKER`: Target pending per worker (default: 1.0)
    /// - `SCALE_DOWN_DELAY_SECONDS`: Scale down delay (default: 300)
    /// - `POLL_INTERVAL_SECONDS`: Poll interval (default: 30)
    ///
    /// # Panics
    ///
    /// Panics if required environment variables are not set or are invalid.
    pub fn from_env() -> Self {
        let orchestrator_url = env::var("ORCHESTRATOR_URL")
            .expect("ORCHESTRATOR_URL must be set");

        let tenant_id = env::var("TENANT_ID")
            .expect("TENANT_ID must be set")
            .parse::<Uuid>()
            .expect("TENANT_ID must be a valid UUID");

        let machine_group = env::var("MACHINE_GROUP")
            .expect("MACHINE_GROUP must be set");

        let deployment_name = env::var("DEPLOYMENT_NAME")
            .expect("DEPLOYMENT_NAME must be set");

        let deployment_namespace = env::var("DEPLOYMENT_NAMESPACE")
            .expect("DEPLOYMENT_NAMESPACE must be set");

        let defaults = ScalingConfig::default();

        let scaling = ScalingConfig {
            min_replicas: env::var("MIN_REPLICAS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.min_replicas),
            max_replicas: env::var("MAX_REPLICAS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.max_replicas),
            target_pending_per_worker: env::var("TARGET_PENDING_PER_WORKER")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.target_pending_per_worker),
            scale_down_delay_seconds: env::var("SCALE_DOWN_DELAY_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.scale_down_delay_seconds),
            poll_interval_seconds: env::var("POLL_INTERVAL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(defaults.poll_interval_seconds),
        };

        Self {
            orchestrator_url,
            tenant_id,
            machine_group,
            deployment_name,
            deployment_namespace,
            scaling,
        }
    }
}
