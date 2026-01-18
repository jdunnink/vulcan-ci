//! HTTP client for communicating with the worker orchestrator.

pub mod dto;

use reqwest::{Client, StatusCode};
use tracing::debug;
use uuid::Uuid;

use crate::config::Config;
use crate::error::{Result, WorkerError};

pub use dto::{
    HeartbeatRequest, HeartbeatResponse, RegisterWorkerRequest, RegisterWorkerResponse,
    WorkRequest, WorkResponse, WorkResultRequest, WorkResultResponse,
};

/// Client for communicating with the worker orchestrator API.
#[derive(Clone)]
pub struct OrchestratorClient {
    client: Client,
    base_url: String,
}

impl OrchestratorClient {
    /// Create a new orchestrator client.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be built.
    pub fn new(config: &Config) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.request_timeout)
            .build()?;

        Ok(Self {
            client,
            base_url: config.orchestrator_url.clone(),
        })
    }

    /// Register this worker with the orchestrator.
    ///
    /// # Errors
    ///
    /// Returns an error if the registration request fails.
    pub async fn register(
        &self,
        tenant_id: Uuid,
        machine_group: Option<String>,
    ) -> Result<RegisterWorkerResponse> {
        let url = format!("{}/workers/register", self.base_url);
        let request = RegisterWorkerRequest {
            tenant_id,
            machine_group,
        };

        debug!(%url, "Registering worker");

        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            let body = response.json::<RegisterWorkerResponse>().await?;
            Ok(body)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(WorkerError::Orchestrator(format!(
                "Registration failed: {status} - {body}"
            )))
        }
    }

    /// Send a heartbeat to the orchestrator.
    ///
    /// # Errors
    ///
    /// Returns an error if the heartbeat request fails.
    pub async fn heartbeat(&self, worker_id: Uuid) -> Result<HeartbeatResponse> {
        let url = format!("{}/workers/heartbeat", self.base_url);
        let request = HeartbeatRequest { worker_id };

        debug!(%url, %worker_id, "Sending heartbeat");

        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            let body = response.json::<HeartbeatResponse>().await?;
            Ok(body)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(WorkerError::Orchestrator(format!(
                "Heartbeat failed: {status} - {body}"
            )))
        }
    }

    /// Request work from the orchestrator.
    ///
    /// Returns `None` if no work is available (204 No Content).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn request_work(&self, worker_id: Uuid) -> Result<Option<WorkResponse>> {
        let url = format!("{}/work/request", self.base_url);
        let request = WorkRequest { worker_id };

        debug!(%url, %worker_id, "Requesting work");

        let response = self.client.post(&url).json(&request).send().await?;

        match response.status() {
            StatusCode::OK => {
                let body = response.json::<WorkResponse>().await?;
                Ok(Some(body))
            }
            StatusCode::NO_CONTENT => Ok(None),
            status => {
                let body = response.text().await.unwrap_or_default();
                Err(WorkerError::Orchestrator(format!(
                    "Work request failed: {status} - {body}"
                )))
            }
        }
    }

    /// Report work execution result to the orchestrator.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails.
    pub async fn report_result(
        &self,
        worker_id: Uuid,
        fragment_id: Uuid,
        success: bool,
        exit_code: Option<i32>,
        error_message: Option<String>,
    ) -> Result<WorkResultResponse> {
        let url = format!("{}/work/result", self.base_url);
        let request = WorkResultRequest {
            worker_id,
            fragment_id,
            success,
            exit_code,
            error_message,
        };

        debug!(%url, %worker_id, %fragment_id, %success, "Reporting result");

        let response = self.client.post(&url).json(&request).send().await?;

        if response.status().is_success() {
            let body = response.json::<WorkResultResponse>().await?;
            Ok(body)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(WorkerError::Orchestrator(format!(
                "Result report failed: {status} - {body}"
            )))
        }
    }
}
