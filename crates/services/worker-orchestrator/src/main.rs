//! Vulcan Worker Orchestrator Service.
//!
//! Manages worker lifecycle and assigns work to available workers.

use std::net::SocketAddr;

use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use vulcan_worker_orchestrator::api::create_router;
use vulcan_worker_orchestrator::orchestrator::health::start_health_monitor;
use vulcan_worker_orchestrator::{AppState, Config};

#[tokio::main]
async fn main() {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "vulcan_worker_orchestrator=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env();
    let addr = config.socket_addr();

    // Create application state
    let state = AppState::new(config);

    // Start the health monitor background task
    start_health_monitor(state.pool.clone(), state.config.clone());

    // Create the router
    let app = create_router(state);

    // Parse socket address
    let socket_addr: SocketAddr = addr.parse().expect("Invalid socket address");

    info!("Starting Vulcan Worker Orchestrator on {}", socket_addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(socket_addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
