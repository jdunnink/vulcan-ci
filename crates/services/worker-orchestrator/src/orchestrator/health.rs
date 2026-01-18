//! Health monitor for detecting dead workers.
//!
//! Background task that runs periodically to:
//! 1. Find workers whose heartbeat is older than the timeout threshold
//! 2. Mark dead workers as Error status
//! 3. Reset their assigned fragments to Pending for retry (if under max attempts)

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use tokio::time::interval;
use tracing::{error, info, warn};

use vulcan_core::models::worker::WorkerStatus;
use vulcan_core::repositories::{
    FragmentRepository, PgFragmentRepository, PgWorkerRepository, WorkerRepository,
};

use crate::config::Config;
use crate::state::DbPool;

/// Start the health monitor background task.
pub fn start_health_monitor(pool: DbPool, config: Arc<Config>) {
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(config.health_check_interval_secs));

        loop {
            ticker.tick().await;

            if let Err(e) = check_worker_health(&pool, &config) {
                error!(error = %e, "Health check failed");
            }
        }
    });
}

/// Check for dead workers and handle them.
fn check_worker_health(pool: &DbPool, config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;

    // Calculate threshold time
    let threshold = Utc::now().naive_utc()
        - chrono::Duration::seconds(config.heartbeat_timeout_secs as i64);

    // Find dead workers
    let dead_workers = {
        let mut worker_repo = PgWorkerRepository::new(&mut conn);
        worker_repo.find_dead_workers(threshold)?
    };

    for worker in dead_workers {
        warn!(
            worker_id = %worker.id,
            last_heartbeat = ?worker.last_heartbeat_at,
            "Worker appears to be dead"
        );

        // Mark worker as error
        {
            let mut worker_repo = PgWorkerRepository::new(&mut conn);
            let mut worker_to_update = worker.clone();
            worker_to_update.status = WorkerStatus::Error;
            worker_repo.update(&worker_to_update)?;
        }

        // If worker had an assigned fragment, reset it for retry
        if let Some(fragment_id) = worker.current_fragment_id {
            let fragment = {
                let mut fragment_repo = PgFragmentRepository::new(&mut conn);
                fragment_repo.find_by_id(fragment_id)?
            };

            if let Some(fragment) = fragment {
                let mut fragment_repo = PgFragmentRepository::new(&mut conn);
                if fragment.attempt < config.max_retry_attempts {
                    info!(
                        fragment_id = %fragment_id,
                        attempt = fragment.attempt,
                        max_attempts = config.max_retry_attempts,
                        "Resetting fragment for retry"
                    );
                    fragment_repo.reset_for_retry(fragment_id)?;
                } else {
                    warn!(
                        fragment_id = %fragment_id,
                        attempt = fragment.attempt,
                        "Fragment exceeded max retry attempts, marking as failed"
                    );
                    fragment_repo.fail_execution(
                        fragment_id,
                        "Worker died and max retry attempts exceeded".to_string(),
                    )?;
                }
            }

            // Clear the worker's assignment
            let mut worker_repo = PgWorkerRepository::new(&mut conn);
            worker_repo.clear_assignment(worker.id)?;
        }
    }

    Ok(())
}
