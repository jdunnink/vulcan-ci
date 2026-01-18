//! Kubernetes deployment scaling module.

use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{Api, Patch, PatchParams},
    Client,
};
use serde_json::json;
use tracing::{info, warn};

use crate::error::{ControllerError, Result};

/// Kubernetes deployment scaler.
pub struct DeploymentScaler {
    api: Api<Deployment>,
    deployment_name: String,
}

impl DeploymentScaler {
    /// Create a new deployment scaler.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Kubernetes namespace
    /// * `deployment_name` - Name of the deployment to scale
    pub async fn new(namespace: &str, deployment_name: String) -> Result<Self> {
        let client = Client::try_default().await?;
        let api: Api<Deployment> = Api::namespaced(client, namespace);

        Ok(Self {
            api,
            deployment_name,
        })
    }

    /// Get the current replica count of the deployment.
    pub async fn get_replicas(&self) -> Result<i32> {
        let deployment = self.api.get(&self.deployment_name).await.map_err(|e| {
            if matches!(e, kube::Error::Api(ref api_err) if api_err.code == 404) {
                ControllerError::DeploymentNotFound {
                    name: self.deployment_name.clone(),
                    namespace: "unknown".to_string(),
                }
            } else {
                ControllerError::Kube(e)
            }
        })?;

        let replicas = deployment
            .spec
            .and_then(|s| s.replicas)
            .unwrap_or(0);

        Ok(replicas)
    }

    /// Scale the deployment to the desired number of replicas.
    ///
    /// # Arguments
    ///
    /// * `replicas` - Desired number of replicas
    pub async fn scale(&self, replicas: i32) -> Result<()> {
        let patch = json!({
            "spec": {
                "replicas": replicas
            }
        });

        let params = PatchParams::apply("vulcan-worker-controller");

        self.api
            .patch(&self.deployment_name, &params, &Patch::Merge(&patch))
            .await?;

        info!(
            deployment = %self.deployment_name,
            replicas = replicas,
            "Scaled deployment"
        );

        Ok(())
    }

    /// Verify the deployment exists.
    pub async fn verify_exists(&self) -> Result<bool> {
        match self.api.get(&self.deployment_name).await {
            Ok(_) => Ok(true),
            Err(kube::Error::Api(ref api_err)) if api_err.code == 404 => {
                warn!(
                    deployment = %self.deployment_name,
                    "Deployment not found"
                );
                Ok(false)
            }
            Err(e) => Err(ControllerError::Kube(e)),
        }
    }
}
