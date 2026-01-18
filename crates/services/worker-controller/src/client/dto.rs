//! Data transfer objects for orchestrator API responses.

use serde::Deserialize;
use uuid::Uuid;

/// Response with queue metrics for scaling decisions.
#[derive(Debug, Deserialize)]
pub struct QueueMetricsResponse {
    /// Number of pending fragments.
    pub pending_fragments: i64,
    /// Number of currently running fragments.
    pub running_fragments: i64,
    /// Number of active workers.
    pub active_workers: i64,
}

/// Response indicating if a worker is busy executing a fragment.
#[derive(Debug, Deserialize)]
pub struct WorkerBusyResponse {
    /// Whether the worker is currently executing a fragment.
    pub busy: bool,
    /// The fragment ID being executed, if any.
    pub fragment_id: Option<Uuid>,
}
