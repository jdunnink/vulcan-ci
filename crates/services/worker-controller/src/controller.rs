//! Main controller reconciliation loop.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;
use tracing::{error, info};

use crate::client::OrchestratorClient;
use crate::config::Config;
use crate::error::Result;
use crate::kubernetes::DeploymentScaler;
use crate::scaler::{calculate_desired_replicas, ScalerState, ScalingConfig};

/// The main worker controller.
pub struct Controller {
    config: Config,
    client: OrchestratorClient,
    scaler: DeploymentScaler,
    state: ScalerState,
}

impl Controller {
    /// Create a new controller.
    ///
    /// # Arguments
    ///
    /// * `config` - Controller configuration
    pub async fn new(config: Config) -> Result<Self> {
        let client = OrchestratorClient::new(config.orchestrator_url.clone());
        let scaler = DeploymentScaler::new(
            &config.deployment_namespace,
            config.deployment_name.clone(),
        )
        .await?;

        Ok(Self {
            config,
            client,
            scaler,
            state: ScalerState::new(),
        })
    }

    /// Run the controller loop.
    ///
    /// # Arguments
    ///
    /// * `shutdown` - Notification for graceful shutdown
    pub async fn run(&mut self, shutdown: Arc<Notify>) -> Result<()> {
        info!(
            tenant_id = %self.config.tenant_id,
            machine_group = %self.config.machine_group,
            deployment = %self.config.deployment_name,
            min_replicas = self.config.scaling.min_replicas,
            max_replicas = self.config.scaling.max_replicas,
            target_pending_per_worker = self.config.scaling.target_pending_per_worker,
            poll_interval_seconds = self.config.scaling.poll_interval_seconds,
            scale_down_delay_seconds = self.config.scaling.scale_down_delay_seconds,
            "Starting worker controller"
        );

        // Verify deployment exists
        if !self.scaler.verify_exists().await? {
            error!(
                deployment = %self.config.deployment_name,
                namespace = %self.config.deployment_namespace,
                "Deployment not found, exiting"
            );
            return Err(crate::error::ControllerError::DeploymentNotFound {
                name: self.config.deployment_name.clone(),
                namespace: self.config.deployment_namespace.clone(),
            });
        }

        let poll_interval = self.config.scaling.poll_interval_seconds;

        // Main reconciliation loop
        loop {
            // Run one reconciliation cycle
            if let Err(e) = self.reconcile().await {
                error!(error = %e, "Reconciliation failed");
            }

            // Wait for next poll interval or shutdown
            tokio::select! {
                () = tokio::time::sleep(Duration::from_secs(poll_interval as u64)) => {}
                () = shutdown.notified() => {
                    info!("Received shutdown signal, stopping controller");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Run one reconciliation cycle.
    async fn reconcile(&mut self) -> Result<()> {
        // Get queue metrics
        let metrics = self
            .client
            .get_queue_metrics(Some(&self.config.machine_group))
            .await?;

        info!(
            pending = metrics.pending_fragments,
            running = metrics.running_fragments,
            active_workers = metrics.active_workers,
            "Got queue metrics"
        );

        // Get current deployment replicas
        let current_replicas = self.scaler.get_replicas().await?;
        self.state.set_current_replicas(current_replicas);

        // Build scaling config from local configuration
        let scaling_config = ScalingConfig {
            min_replicas: self.config.scaling.min_replicas,
            max_replicas: self.config.scaling.max_replicas,
            target_pending_per_worker: self.config.scaling.target_pending_per_worker,
        };

        let desired_replicas = calculate_desired_replicas(&scaling_config, metrics.pending_fragments);

        info!(
            current = current_replicas,
            desired = desired_replicas,
            "Calculated replica count"
        );

        // Check if scaling is needed
        let scale_down_delay = self.config.scaling.scale_down_delay_seconds;
        if let Some(new_replicas) = self.state.should_scale(desired_replicas, scale_down_delay) {
            // Perform scaling
            self.scaler.scale(new_replicas).await?;

            // Record scale-down for cooldown tracking
            if new_replicas < current_replicas {
                self.state.record_scale_down();
            }

            self.state.set_current_replicas(new_replicas);

            info!(
                from = current_replicas,
                to = new_replicas,
                "Scaled deployment"
            );
        } else if desired_replicas != current_replicas {
            info!(
                current = current_replicas,
                desired = desired_replicas,
                "Scale-down blocked by cooldown"
            );
        }

        Ok(())
    }
}
