//! Vulcan Worker Service.
//!
//! Executes individual chain fragments and reports results.

use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use vulcan_worker::config::Config;
use vulcan_worker::worker::Worker;

#[tokio::main]
async fn main() {
    // Load .env file if present
    let _ = dotenvy::dotenv();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vulcan_worker=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Vulcan Worker");

    // Load configuration
    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to load configuration");
            std::process::exit(1);
        }
    };

    info!(
        orchestrator_url = %config.orchestrator_url,
        tenant_id = %config.tenant_id,
        worker_group = ?config.worker_group,
        heartbeat_interval_secs = config.heartbeat_interval.as_secs(),
        poll_interval_secs = config.poll_interval.as_secs(),
        script_timeout_secs = config.script_timeout.as_secs(),
        "Configuration loaded"
    );

    // Create worker
    let mut worker = match Worker::new(config) {
        Ok(w) => w,
        Err(e) => {
            error!(error = %e, "Failed to create worker");
            std::process::exit(1);
        }
    };

    // Get shutdown handle for Ctrl+C
    let shutdown = worker.shutdown_handle();

    // Spawn Ctrl+C handler
    tokio::spawn(async move {
        if let Err(e) = signal::ctrl_c().await {
            error!(error = %e, "Failed to listen for Ctrl+C");
            return;
        }
        info!("Received Ctrl+C, initiating graceful shutdown");
        shutdown.notify_waiters();
    });

    // Run worker
    if let Err(e) = worker.run().await {
        error!(error = %e, "Worker error");
        std::process::exit(1);
    }

    info!("Vulcan Worker shutdown complete");
}
