//! Orchestrator HTTP client module.

pub mod dto;

use reqwest::Client;

use crate::error::Result;
use dto::QueueMetricsResponse;

/// Client for communicating with the orchestrator service.
pub struct OrchestratorClient {
    client: Client,
    base_url: String,
}

impl OrchestratorClient {
    /// Create a new orchestrator client.
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    /// Get queue metrics for scaling decisions.
    ///
    /// # Arguments
    ///
    /// * `machine_group` - Optional machine group to filter metrics
    pub async fn get_queue_metrics(
        &self,
        machine_group: Option<&str>,
    ) -> Result<QueueMetricsResponse> {
        let mut url = format!("{}/queue/metrics", self.base_url);

        if let Some(group) = machine_group {
            url = format!("{}?machine_group={}", url, group);
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<QueueMetricsResponse>()
            .await?;

        Ok(response)
    }
}
