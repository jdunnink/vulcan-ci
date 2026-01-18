//! Worker controller service entry point.

use std::sync::Arc;

use tokio::sync::Notify;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use vulcan_worker_controller::{Config, Controller};

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,vulcan_worker_controller=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables from .env file if present
    if dotenvy::dotenv().is_ok() {
        info!("Loaded .env file");
    }

    // Load configuration
    let config = Config::from_env();

    info!(
        tenant_id = %config.tenant_id,
        machine_group = %config.machine_group,
        deployment = %config.deployment_name,
        namespace = %config.deployment_namespace,
        "Starting vulcan-worker-controller"
    );

    // Create shutdown notification
    let shutdown = Arc::new(Notify::new());
    let shutdown_clone = Arc::clone(&shutdown);

    // Setup Ctrl+C handler
    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C handler");
        info!("Received CTRL+C, initiating shutdown");
        shutdown_clone.notify_one();
    });

    // Create and run controller
    let mut controller = match Controller::new(config).await {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "Failed to create controller");
            std::process::exit(1);
        }
    };

    if let Err(e) = controller.run(shutdown).await {
        error!(error = %e, "Controller error");
        std::process::exit(1);
    }

    info!("Worker controller stopped");
}
