//! HTTP request handlers.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use diesel::pg::PgConnection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use vulcan_chain_parser::{ChainParserService, ImportFetcher, ParseError, Result as ParseResult, WorkflowContext};
use vulcan_core::models::chain::TriggerType;
use vulcan_core::repositories::{ChainRepository, FragmentRepository, PgChainRepository, PgFragmentRepository};

use crate::error::ApiError;

/// Shared application state.
pub struct AppState {
    /// Database connection (wrapped in Mutex for thread-safe access).
    pub db: Mutex<PgConnection>,
}

/// No-op fetcher that rejects all imports.
///
/// In the API context, imports should be pre-resolved or disabled.
struct NoOpFetcher;

impl ImportFetcher for NoOpFetcher {
    fn fetch(&self, url: &str) -> ParseResult<String> {
        Err(ParseError::FetchFailed {
            url: url.to_string(),
            reason: "imports are not supported in API mode".to_string(),
        })
    }
}

/// Request body for parsing a workflow.
#[derive(Debug, Deserialize)]
pub struct ParseRequest {
    /// The KDL workflow content to parse.
    pub content: String,

    /// Tenant ID for the chain.
    pub tenant_id: Uuid,

    /// Optional source file path for reference.
    #[serde(default)]
    pub source_file_path: Option<String>,

    /// Optional repository URL.
    #[serde(default)]
    pub repository_url: Option<String>,

    /// Optional commit SHA.
    #[serde(default)]
    pub commit_sha: Option<String>,

    /// Optional branch name.
    #[serde(default)]
    pub branch: Option<String>,

    /// Optional trigger type.
    #[serde(default)]
    pub trigger: Option<String>,

    /// Optional trigger reference.
    #[serde(default)]
    pub trigger_ref: Option<String>,
}

/// Response body for successful parse.
#[derive(Debug, Serialize)]
pub struct ParseResponse {
    /// The created chain ID.
    pub chain_id: Uuid,

    /// Number of fragments created.
    pub fragment_count: usize,

    /// Message.
    pub message: String,
}

/// Health check endpoint.
pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Parse and store a workflow.
///
/// POST /parse
pub async fn parse_workflow(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ParseRequest>,
) -> Result<Json<ParseResponse>, ApiError> {
    // Build workflow context
    let mut context = WorkflowContext::new(request.tenant_id);

    if let Some(ref path) = request.source_file_path {
        context = context.with_source(path.clone());
    }
    if let Some(ref url) = request.repository_url {
        context = context.with_repository(url.clone());
    }
    if let Some(ref sha) = request.commit_sha {
        context = context.with_commit(sha.clone());
    }
    if let Some(ref branch) = request.branch {
        context = context.with_branch(branch.clone());
    }
    if let Some(ref trigger) = request.trigger {
        let trigger_type = parse_trigger_type(trigger)?;
        context = context.with_trigger(trigger_type, request.trigger_ref.clone());
    }

    // Parse the workflow
    let service = ChainParserService::new(NoOpFetcher);
    let parsed = if request.trigger.is_some() {
        service.parse(&request.content, &context)?
    } else {
        service.parse_without_trigger_validation(&request.content, &context)?
    };

    // Store in database
    let (chain_id, fragment_count) = {
        let mut conn = state
            .db
            .lock()
            .map_err(|e| ApiError::Internal(format!("failed to acquire db lock: {e}")))?;

        let mut chain_repo = PgChainRepository::new(&mut conn);
        let chain = chain_repo.create(parsed.chain)?;

        let mut fragment_repo = PgFragmentRepository::new(chain_repo.conn());
        let fragments = fragment_repo.create_many(parsed.fragments)?;

        (chain.id, fragments.len())
    };

    Ok(Json(ParseResponse {
        chain_id,
        fragment_count,
        message: "Workflow parsed and stored successfully".to_string(),
    }))
}

/// Parse trigger type from string.
fn parse_trigger_type(s: &str) -> Result<TriggerType, ApiError> {
    match s.to_lowercase().as_str() {
        "push" => Ok(TriggerType::Push),
        "pull_request" => Ok(TriggerType::PullRequest),
        "tag" => Ok(TriggerType::Tag),
        "schedule" => Ok(TriggerType::Schedule),
        "manual" => Ok(TriggerType::Manual),
        _ => Err(ApiError::InvalidRequest(format!("invalid trigger type: {s}"))),
    }
}
