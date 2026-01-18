//! Worker state machine and main loop.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::client::OrchestratorClient;
use crate::config::Config;
use crate::error::{Result, WorkerError};
use crate::executor::Executor;

/// Maximum backoff duration for retries.
const MAX_BACKOFF_SECS: u64 = 60;

/// Initial backoff duration for retries.
const INITIAL_BACKOFF_SECS: u64 = 1;

/// Worker that connects to the orchestrator and executes work.
pub struct Worker {
    config: Config,
    client: OrchestratorClient,
    executor: Executor,
    worker_id: Option<Uuid>,
    shutdown: Arc<Notify>,
}

impl Worker {
    /// Create a new worker.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(config: Config) -> Result<Self> {
        let client = OrchestratorClient::new(&config)?;
        let executor = Executor::new(config.script_timeout);

        Ok(Self {
            config,
            client,
            executor,
            worker_id: None,
            shutdown: Arc::new(Notify::new()),
        })
    }

    /// Get the shutdown notifier for graceful shutdown.
    #[must_use]
    pub fn shutdown_handle(&self) -> Arc<Notify> {
        Arc::clone(&self.shutdown)
    }

    /// Run the worker main loop.
    ///
    /// This will:
    /// 1. Register with the orchestrator (with retry)
    /// 2. Start the heartbeat task
    /// 3. Start the work loop
    ///
    /// # Errors
    ///
    /// Returns an error if a fatal error occurs.
    pub async fn run(&mut self) -> Result<()> {
        // Register with retry
        self.register_with_retry().await?;

        let worker_id = self.worker_id.ok_or(WorkerError::NotRegistered)?;
        info!(%worker_id, "Worker registered and running");

        // Spawn heartbeat task
        let heartbeat_handle = self.spawn_heartbeat_task(worker_id);

        // Run work loop
        let work_result = self.work_loop(worker_id).await;

        // Cancel heartbeat task
        heartbeat_handle.abort();

        work_result
    }

    /// Register with the orchestrator, retrying with exponential backoff.
    async fn register_with_retry(&mut self) -> Result<()> {
        let mut backoff = Duration::from_secs(INITIAL_BACKOFF_SECS);

        loop {
            match self
                .client
                .register(self.config.tenant_id, self.config.worker_group.clone())
                .await
            {
                Ok(response) => {
                    self.worker_id = Some(response.worker_id);
                    info!(
                        worker_id = %response.worker_id,
                        status = %response.status,
                        "Successfully registered with orchestrator"
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        error = %e,
                        backoff_secs = backoff.as_secs(),
                        "Failed to register, retrying"
                    );

                    // Check for shutdown
                    tokio::select! {
                        () = sleep(backoff) => {}
                        () = self.shutdown.notified() => {
                            info!("Shutdown requested during registration");
                            return Err(WorkerError::Orchestrator("Shutdown requested".to_string()));
                        }
                    }

                    backoff = std::cmp::min(backoff * 2, Duration::from_secs(MAX_BACKOFF_SECS));
                }
            }
        }
    }

    /// Spawn the heartbeat background task.
    fn spawn_heartbeat_task(&self, worker_id: Uuid) -> tokio::task::JoinHandle<()> {
        let client = self.client.clone();
        let interval = self.config.heartbeat_interval;
        let shutdown = Arc::clone(&self.shutdown);

        tokio::spawn(async move {
            let mut backoff = Duration::from_secs(INITIAL_BACKOFF_SECS);

            loop {
                tokio::select! {
                    () = sleep(interval) => {
                        match client.heartbeat(worker_id).await {
                            Ok(_) => {
                                debug!(%worker_id, "Heartbeat sent");
                                backoff = Duration::from_secs(INITIAL_BACKOFF_SECS);
                            }
                            Err(e) => {
                                warn!(
                                    %worker_id,
                                    error = %e,
                                    "Heartbeat failed"
                                );
                                // Use exponential backoff for failed heartbeats
                                sleep(backoff).await;
                                backoff = std::cmp::min(backoff * 2, Duration::from_secs(MAX_BACKOFF_SECS));
                            }
                        }
                    }
                    () = shutdown.notified() => {
                        info!(%worker_id, "Heartbeat task shutting down");
                        return;
                    }
                }
            }
        })
    }

    /// Main work loop: request work, execute, report results.
    async fn work_loop(&self, worker_id: Uuid) -> Result<()> {
        let mut backoff = Duration::from_secs(INITIAL_BACKOFF_SECS);

        loop {
            tokio::select! {
                () = self.shutdown.notified() => {
                    info!(%worker_id, "Work loop shutting down");
                    return Ok(());
                }
                result = self.work_cycle(worker_id) => {
                    match result {
                        Ok(had_work) => {
                            backoff = Duration::from_secs(INITIAL_BACKOFF_SECS);
                            if !had_work {
                                // No work available, wait before polling again
                                sleep(self.config.poll_interval).await;
                            }
                        }
                        Err(e) => {
                            error!(%worker_id, error = %e, "Work cycle error");
                            sleep(backoff).await;
                            backoff = std::cmp::min(backoff * 2, Duration::from_secs(MAX_BACKOFF_SECS));
                        }
                    }
                }
            }
        }
    }

    /// Single work cycle: request work, execute if available, report result.
    ///
    /// Returns `true` if work was executed, `false` if no work was available.
    async fn work_cycle(&self, worker_id: Uuid) -> Result<bool> {
        // Request work
        let work = self.client.request_work(worker_id).await?;

        let Some(work) = work else {
            debug!(%worker_id, "No work available");
            return Ok(false);
        };

        info!(
            %worker_id,
            fragment_id = %work.fragment_id,
            chain_id = %work.chain_id,
            attempt = work.attempt,
            "Received work"
        );

        // Execute the script
        let output = if let Some(script) = &work.run_script {
            self.executor.execute(work.fragment_id, script).await?
        } else {
            warn!(
                %worker_id,
                fragment_id = %work.fragment_id,
                "Fragment has no run_script"
            );
            crate::executor::ExecutionOutput::new(
                String::new(),
                "No script to execute".to_string(),
                1,
            )
        };

        // Report result
        self.client
            .report_result(
                worker_id,
                work.fragment_id,
                output.success,
                Some(output.exit_code),
                output.error_message(),
            )
            .await?;

        info!(
            %worker_id,
            fragment_id = %work.fragment_id,
            success = output.success,
            exit_code = output.exit_code,
            "Work completed and reported"
        );

        Ok(true)
    }
}
