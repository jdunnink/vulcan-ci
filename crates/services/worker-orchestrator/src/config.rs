//! Configuration for the worker orchestrator service.

use std::env;

/// Configuration for the worker orchestrator.
#[derive(Debug, Clone)]
pub struct Config {
    /// Database connection URL.
    pub database_url: String,
    /// Host to bind the HTTP server to.
    pub host: String,
    /// Port to bind the HTTP server to.
    pub port: u16,
    /// Heartbeat timeout in seconds (workers not heard from in this time are considered dead).
    pub heartbeat_timeout_secs: u64,
    /// How often to run the health check in seconds.
    pub health_check_interval_secs: u64,
    /// Maximum retry attempts for failed fragments.
    pub max_retry_attempts: i32,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Panics
    /// Panics if required environment variables are not set.
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3002".to_string())
                .parse()
                .expect("PORT must be a valid number"),
            heartbeat_timeout_secs: env::var("HEARTBEAT_TIMEOUT_SECS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .expect("HEARTBEAT_TIMEOUT_SECS must be a valid number"),
            health_check_interval_secs: env::var("HEALTH_CHECK_INTERVAL_SECS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("HEALTH_CHECK_INTERVAL_SECS must be a valid number"),
            max_retry_attempts: env::var("MAX_RETRY_ATTEMPTS")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .expect("MAX_RETRY_ATTEMPTS must be a valid number"),
        }
    }

    /// Returns the socket address to bind to.
    pub fn socket_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
