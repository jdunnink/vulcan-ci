//! Data transfer objects for orchestrator API communication.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Worker Registration
// ============================================================================

/// Request to register a worker with the orchestrator.
#[derive(Debug, Serialize)]
pub struct RegisterWorkerRequest {
    /// Tenant ID the worker belongs to.
    pub tenant_id: Uuid,
    /// Machine group this worker belongs to (optional).
    pub machine_group: Option<String>,
}

/// Response from worker registration.
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Serialize)]
pub struct HeartbeatRequest {
    /// Worker ID sending the heartbeat.
    pub worker_id: Uuid,
}

/// Response from heartbeat.
#[derive(Debug, Deserialize)]
pub struct HeartbeatResponse {
    /// Acknowledgment message.
    pub status: String,
    /// Server timestamp.
    pub timestamp: NaiveDateTime,
}

// ============================================================================
// Work Request
// ============================================================================

/// Request for work from the orchestrator.
#[derive(Debug, Serialize)]
pub struct WorkRequest {
    /// Worker ID requesting work.
    pub worker_id: Uuid,
}

/// Response with assigned work.
#[derive(Debug, Deserialize)]
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

/// Request to report work execution result.
#[derive(Debug, Serialize)]
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
#[derive(Debug, Deserialize)]
pub struct WorkResultResponse {
    /// Acknowledgment status.
    pub status: String,
    /// Fragment status after update.
    pub fragment_status: String,
}
