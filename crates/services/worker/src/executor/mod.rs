//! Script execution module.

pub mod output;

use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub use output::ExecutionOutput;

use crate::error::Result;

/// Script executor that runs shell scripts with timeout enforcement.
#[derive(Debug, Clone)]
pub struct Executor {
    /// Timeout for script execution.
    timeout: Duration,
}

impl Executor {
    /// Create a new executor with the given timeout.
    #[must_use]
    pub fn new(timeout: Duration) -> Self {
        Self { timeout }
    }

    /// Execute a script and return the output.
    ///
    /// The script is executed via `/bin/sh -c`.
    pub async fn execute(&self, fragment_id: Uuid, script: &str) -> Result<ExecutionOutput> {
        info!(%fragment_id, "Executing script");
        debug!(%fragment_id, script = %script, "Script content");

        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        // Take stdout and stderr handles before waiting
        let stdout_handle = child.stdout.take();
        let stderr_handle = child.stderr.take();

        let result = timeout(self.timeout, child.wait()).await;

        match result {
            Ok(Ok(status)) => {
                // Process completed, read output
                let stdout = if let Some(mut handle) = stdout_handle {
                    use tokio::io::AsyncReadExt;
                    let mut buf = Vec::new();
                    let _ = handle.read_to_end(&mut buf).await;
                    String::from_utf8_lossy(&buf).to_string()
                } else {
                    String::new()
                };

                let stderr = if let Some(mut handle) = stderr_handle {
                    use tokio::io::AsyncReadExt;
                    let mut buf = Vec::new();
                    let _ = handle.read_to_end(&mut buf).await;
                    String::from_utf8_lossy(&buf).to_string()
                } else {
                    String::new()
                };

                let exit_code = status.code().unwrap_or(-1);

                info!(
                    %fragment_id,
                    %exit_code,
                    stdout_len = stdout.len(),
                    stderr_len = stderr.len(),
                    "Script completed"
                );

                Ok(ExecutionOutput::new(stdout, stderr, exit_code))
            }
            Ok(Err(e)) => {
                warn!(%fragment_id, error = %e, "Script execution error");
                Ok(ExecutionOutput::new(
                    String::new(),
                    e.to_string(),
                    -1,
                ))
            }
            Err(_) => {
                warn!(
                    %fragment_id,
                    timeout_secs = self.timeout.as_secs(),
                    "Script execution timed out"
                );

                // Kill the process (kill_on_drop will handle this when child is dropped)
                if let Err(e) = child.kill().await {
                    warn!(%fragment_id, error = %e, "Failed to kill timed out process");
                }

                // Try to read any partial output
                let stdout = if let Some(mut handle) = stdout_handle {
                    use tokio::io::AsyncReadExt;
                    let mut buf = Vec::new();
                    let _ = handle.read_to_end(&mut buf).await;
                    String::from_utf8_lossy(&buf).to_string()
                } else {
                    String::new()
                };

                let stderr = if let Some(mut handle) = stderr_handle {
                    use tokio::io::AsyncReadExt;
                    let mut buf = Vec::new();
                    let _ = handle.read_to_end(&mut buf).await;
                    String::from_utf8_lossy(&buf).to_string()
                } else {
                    String::new()
                };

                Ok(ExecutionOutput::timeout(stdout, stderr))
            }
        }
    }
}
