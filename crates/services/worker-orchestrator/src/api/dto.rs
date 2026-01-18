//! Data transfer objects for the API.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Worker Registration
// ============================================================================

/// Request to register a new worker.
#[derive(Debug, Deserialize)]
pub struct RegisterWorkerRequest {
    /// Tenant ID the worker belongs to.
    pub tenant_id: Uuid,
    /// Machine group this worker belongs to (optional).
    pub machine_group: Option<String>,
}

/// Response after registering a worker.
#[derive(Debug, Serialize)]
pub struct RegisterWorkerResponse {
    /// The assigned worker ID.
    pub worker_id: Uuid,
    /// Current status of the worker.
    pub status: String,
}

// ============================================================================
// Heartbeat
// ============================================================================

/// Request to send a heartbeat.
#[derive(Debug, Deserialize)]
pub struct HeartbeatRequest {
    /// Worker ID sending the heartbeat.
    pub worker_id: Uuid,
}

/// Response after a heartbeat.
#[derive(Debug, Serialize)]
pub struct HeartbeatResponse {
    /// Acknowledgment message.
    pub status: String,
    /// Server timestamp.
    pub timestamp: NaiveDateTime,
}

// ============================================================================
// Work Request
// ============================================================================

/// Request for work from a worker.
#[derive(Debug, Deserialize)]
pub struct WorkRequest {
    /// Worker ID requesting work.
    pub worker_id: Uuid,
}

/// Response with assigned work.
#[derive(Debug, Serialize)]
pub struct WorkResponse {
    /// The assigned fragment ID.
    pub fragment_id: Uuid,
    /// The chain this fragment belongs to.
    pub chain_id: Uuid,
    /// Script to execute (for inline fragments).
    pub run_script: Option<String>,
    /// Current attempt number.
    pub attempt: i32,
}

// ============================================================================
// Work Result
// ============================================================================

/// Request to report work result.
#[derive(Debug, Deserialize)]
pub struct WorkResultRequest {
    /// Worker ID reporting the result.
    pub worker_id: Uuid,
    /// Fragment ID that was executed.
    pub fragment_id: Uuid,
    /// Whether execution succeeded.
    pub success: bool,
    /// Exit code from execution.
    pub exit_code: Option<i32>,
    /// Error message if failed.
    pub error_message: Option<String>,
}

/// Response after reporting work result.
#[derive(Debug, Serialize)]
pub struct WorkResultResponse {
    /// Acknowledgment status.
    pub status: String,
    /// Fragment status after update.
    pub fragment_status: String,
}

// ============================================================================
// Health Check
// ============================================================================

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Service status.
    pub status: String,
    /// Service name.
    pub service: String,
}

// ============================================================================
// Queue Metrics (for worker-controller scaling decisions)
// ============================================================================

/// Response with queue metrics for scaling decisions.
#[derive(Debug, Serialize)]
pub struct QueueMetricsResponse {
    /// Number of pending fragments.
    pub pending_fragments: i64,
    /// Number of currently running fragments.
    pub running_fragments: i64,
    /// Number of active workers.
    pub active_workers: i64,
}

// ============================================================================
// Worker Busy Check (for preStop hook)
// ============================================================================

/// Response indicating if a worker is busy executing a fragment.
#[derive(Debug, Serialize)]
pub struct WorkerBusyResponse {
    /// Whether the worker is currently executing a fragment.
    pub busy: bool,
    /// The fragment ID being executed, if any.
    pub fragment_id: Option<Uuid>,
}
