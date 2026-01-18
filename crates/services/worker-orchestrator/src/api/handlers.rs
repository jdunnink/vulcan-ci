//! HTTP request handlers for the worker orchestrator API.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use tracing::{info, warn};
use uuid::Uuid;

use axum::extract::{Path, Query};

use vulcan_core::models::worker::NewWorker;
use vulcan_core::repositories::{
    ChainRepository, FragmentRepository, PgChainRepository, PgFragmentRepository,
    PgWorkerRepository, WorkerRepository,
};

use crate::api::dto::{
    HeartbeatRequest, HeartbeatResponse, HealthResponse, QueueMetricsResponse,
    RegisterWorkerRequest, RegisterWorkerResponse, WorkRequest, WorkResponse, WorkResultRequest,
    WorkResultResponse, WorkerBusyResponse,
};
use crate::error::{OrchestratorError, Result};
use crate::orchestrator::scheduler::Scheduler;
use crate::state::AppState;

/// Health check endpoint.
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: "vulcan-worker-orchestrator".to_string(),
    })
}

/// Register a new worker.
pub async fn register_worker(
    State(state): State<AppState>,
    Json(request): Json<RegisterWorkerRequest>,
) -> Result<Json<RegisterWorkerResponse>> {
    let mut conn = state.get_conn()?;
    let mut repo = PgWorkerRepository::new(&mut conn);

    let new_worker = NewWorker::new(request.tenant_id)
        .with_heartbeat(Utc::now().naive_utc());

    let new_worker = if let Some(group) = request.machine_group {
        new_worker.with_machine_group(group)
    } else {
        new_worker
    };

    let worker = repo.create(new_worker)?;

    info!(worker_id = %worker.id, tenant_id = %worker.tenant_id, "Worker registered");

    Ok(Json(RegisterWorkerResponse {
        worker_id: worker.id,
        status: format!("{:?}", worker.status),
    }))
}

/// Handle worker heartbeat.
pub async fn heartbeat(
    State(state): State<AppState>,
    Json(request): Json<HeartbeatRequest>,
) -> Result<Json<HeartbeatResponse>> {
    let mut conn = state.get_conn()?;
    let mut repo = PgWorkerRepository::new(&mut conn);

    // Verify worker exists
    let worker = repo
        .find_by_id(request.worker_id)?
        .ok_or(OrchestratorError::WorkerNotFound(request.worker_id))?;

    // Update heartbeat
    repo.update_heartbeat(worker.id)?;

    let now = Utc::now().naive_utc();

    Ok(Json(HeartbeatResponse {
        status: "ok".to_string(),
        timestamp: now,
    }))
}

/// Worker requests work to execute.
///
/// Uses optimistic locking to atomically claim work, preventing race conditions
/// when thousands of workers request work simultaneously.
pub async fn request_work(
    State(state): State<AppState>,
    Json(request): Json<WorkRequest>,
) -> Result<(StatusCode, Json<Option<WorkResponse>>)> {
    let mut conn = state.get_conn()?;

    // Get the worker
    let worker = {
        let mut repo = PgWorkerRepository::new(&mut conn);
        repo.find_by_id(request.worker_id)?
            .ok_or(OrchestratorError::WorkerNotFound(request.worker_id))?
    };

    // Use the scheduler to find and atomically claim work
    // This uses optimistic locking: if another worker claims the fragment first,
    // the scheduler will try the next eligible fragment
    let scheduler = Scheduler::new(&mut conn);
    let fragment = scheduler.find_and_claim_work(&worker)?;

    match fragment {
        Some(fragment) => {
            // Fragment is already claimed (status=Running, assigned_worker_id set)
            // Just need to update worker's current_fragment_id
            let fragment_id = fragment.id;
            let chain_id = fragment.chain_id;
            let run_script = fragment.run_script.clone();
            let attempt = fragment.attempt;
            let worker_id = worker.id;

            {
                let mut worker_repo = PgWorkerRepository::new(&mut conn);
                worker_repo.assign_fragment(worker_id, fragment_id)?;
            }

            info!(
                worker_id = %worker_id,
                fragment_id = %fragment_id,
                "Assigned fragment to worker"
            );

            Ok((
                StatusCode::OK,
                Json(Some(WorkResponse {
                    fragment_id,
                    chain_id,
                    run_script,
                    attempt,
                })),
            ))
        }
        None => Ok((StatusCode::NO_CONTENT, Json(None))),
    }
}

/// Worker reports execution result.
pub async fn report_result(
    State(state): State<AppState>,
    Json(request): Json<WorkResultRequest>,
) -> Result<Json<WorkResultResponse>> {
    let mut conn = state.get_conn()?;

    // Verify worker exists
    {
        let mut repo = PgWorkerRepository::new(&mut conn);
        repo.find_by_id(request.worker_id)?
            .ok_or(OrchestratorError::WorkerNotFound(request.worker_id))?;
    }

    // Update fragment status
    let fragment = {
        let mut repo = PgFragmentRepository::new(&mut conn);

        if request.success {
            let exit_code = request.exit_code.unwrap_or(0);
            repo.complete_execution(request.fragment_id, exit_code)?
        } else {
            let error = request
                .error_message
                .unwrap_or_else(|| "Unknown error".to_string());
            repo.fail_execution(request.fragment_id, error)?
        }
    };

    // Clear worker assignment
    {
        let mut repo = PgWorkerRepository::new(&mut conn);
        repo.clear_assignment(request.worker_id)?;
    }

    info!(
        worker_id = %request.worker_id,
        fragment_id = %request.fragment_id,
        success = request.success,
        "Fragment execution completed"
    );

    // Check if chain is complete
    check_chain_completion(&mut conn, fragment.chain_id)?;

    Ok(Json(WorkResultResponse {
        status: "ok".to_string(),
        fragment_status: format!("{:?}", fragment.status),
    }))
}

/// Check if all fragments in a chain are complete and update chain status.
fn check_chain_completion(
    conn: &mut diesel::PgConnection,
    chain_id: Uuid,
) -> Result<()> {
    let mut fragment_repo = PgFragmentRepository::new(conn);
    let fragments = fragment_repo.find_by_chain(chain_id)?;

    let all_complete = fragments
        .iter()
        .all(|f| f.status.is_terminal());

    if all_complete {
        let any_failed = fragments
            .iter()
            .any(|f| !f.status.is_success());

        // Need a new connection scope for chain repo
        let mut chain_repo = PgChainRepository::new(conn);

        if any_failed {
            chain_repo.mark_failed(chain_id)?;
            warn!(chain_id = %chain_id, "Chain failed");
        } else {
            chain_repo.mark_completed(chain_id)?;
            info!(chain_id = %chain_id, "Chain completed successfully");
        }
    }

    Ok(())
}

// ============================================================================
// Queue Metrics (for worker-controller scaling decisions)
// ============================================================================

/// Query parameters for queue metrics endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct QueueMetricsQuery {
    /// Filter by machine group (optional).
    pub machine_group: Option<String>,
}

/// Get queue metrics for scaling decisions.
pub async fn queue_metrics(
    State(state): State<AppState>,
    Query(query): Query<QueueMetricsQuery>,
) -> Result<Json<QueueMetricsResponse>> {
    let mut conn = state.get_conn()?;

    let machine_group = query.machine_group.as_deref();

    let pending_fragments = {
        let mut repo = PgFragmentRepository::new(&mut conn);
        repo.count_pending_by_machine(machine_group)?
    };

    let running_fragments = {
        let mut repo = PgFragmentRepository::new(&mut conn);
        repo.count_running_by_machine(machine_group)?
    };

    let active_workers = {
        let mut repo = PgWorkerRepository::new(&mut conn);
        repo.count_active_by_machine_group(machine_group)?
    };

    Ok(Json(QueueMetricsResponse {
        pending_fragments,
        running_fragments,
        active_workers,
    }))
}

// ============================================================================
// Worker Busy Check (for preStop hook)
// ============================================================================

/// Check if a worker is currently busy executing a fragment.
pub async fn worker_busy(
    State(state): State<AppState>,
    Path(worker_id): Path<Uuid>,
) -> Result<Json<WorkerBusyResponse>> {
    let mut conn = state.get_conn()?;
    let mut repo = PgWorkerRepository::new(&mut conn);

    let fragment_id = repo.is_busy(worker_id)?;

    Ok(Json(WorkerBusyResponse {
        busy: fragment_id.is_some(),
        fragment_id,
    }))
}
