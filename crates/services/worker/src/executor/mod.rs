//! Script execution module with bubblewrap sandboxing.

pub mod output;

use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, info, warn};
use uuid::Uuid;

pub use output::ExecutionOutput;

use crate::config::SandboxConfig;
use crate::error::Result;

/// Script executor that runs shell scripts with timeout enforcement.
///
/// When sandboxing is enabled, scripts run inside a bubblewrap (bwrap) sandbox
/// with the following isolation:
/// - New PID namespace (process can't see host processes)
/// - New mount namespace (isolated filesystem view)
/// - New network namespace (no network if disabled)
/// - Read-only root filesystem
/// - Writable scratch directory for execution
#[derive(Debug, Clone)]
pub struct Executor {
    /// Timeout for script execution.
    timeout: Duration,
    /// Sandbox configuration.
    sandbox: SandboxConfig,
}

impl Executor {
    /// Create a new executor with the given timeout and sandbox config.
    #[must_use]
    pub fn new(timeout: Duration, sandbox: SandboxConfig) -> Self {
        Self { timeout, sandbox }
    }

    /// Execute a script and return the output.
    ///
    /// If sandboxing is enabled, the script runs inside bubblewrap.
    /// Otherwise, it runs directly via `/bin/sh -c`.
    pub async fn execute(&self, fragment_id: Uuid, script: &str) -> Result<ExecutionOutput> {
        info!(%fragment_id, sandbox_enabled = self.sandbox.enabled, "Executing script");
        debug!(%fragment_id, script = %script, "Script content");

        let mut child = if self.sandbox.enabled {
            self.spawn_sandboxed(script)?
        } else {
            self.spawn_direct(script)?
        };

        // Take stdout and stderr handles before waiting
        let stdout_handle = child.stdout.take();
        let stderr_handle = child.stderr.take();

        let result = timeout(self.timeout, child.wait()).await;

        match result {
            Ok(Ok(status)) => {
                // Process completed, read output
                let stdout = read_handle(stdout_handle).await;
                let stderr = read_handle_stderr(stderr_handle).await;

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
                let stdout = read_handle(stdout_handle).await;
                let stderr = read_handle_stderr(stderr_handle).await;

                Ok(ExecutionOutput::timeout(stdout, stderr))
            }
        }
    }

    /// Spawn script directly without sandboxing.
    fn spawn_direct(&self, script: &str) -> std::io::Result<tokio::process::Child> {
        Command::new("/bin/sh")
            .arg("-c")
            .arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
    }

    /// Spawn script inside bubblewrap sandbox.
    ///
    /// Bubblewrap provides namespace-based isolation:
    /// - `--unshare-pid`: New PID namespace
    /// - `--unshare-net`: New network namespace (if network disabled)
    /// - `--unshare-uts`: New UTS namespace (hostname isolation)
    /// - `--unshare-ipc`: New IPC namespace
    /// - `--die-with-parent`: Kill sandbox if parent dies
    /// - `--new-session`: New session to prevent terminal access
    /// - `--ro-bind`: Read-only filesystem binds
    /// - `--bind`: Writable bind for scratch directory
    /// - `--dev /dev`: Minimal /dev
    /// - `--proc /proc`: Process filesystem
    /// - `--tmpfs /tmp`: Temporary filesystem
    fn spawn_sandboxed(&self, script: &str) -> std::io::Result<tokio::process::Child> {
        let mut cmd = Command::new("bwrap");

        // Namespace isolation
        cmd.arg("--unshare-pid")
            .arg("--unshare-uts")
            .arg("--unshare-ipc");

        // Network isolation (disable if not allowed)
        if !self.sandbox.network {
            cmd.arg("--unshare-net");
        }

        // Security settings
        cmd.arg("--die-with-parent")
            .arg("--new-session");

        // Filesystem setup - read-only root
        cmd.arg("--ro-bind").arg("/usr").arg("/usr")
            .arg("--ro-bind").arg("/lib").arg("/lib")
            .arg("--ro-bind").arg("/lib64").arg("/lib64")
            .arg("--ro-bind").arg("/bin").arg("/bin")
            .arg("--ro-bind").arg("/sbin").arg("/sbin");

        // Optional: etc for basic system config (read-only)
        cmd.arg("--ro-bind").arg("/etc/passwd").arg("/etc/passwd")
            .arg("--ro-bind").arg("/etc/group").arg("/etc/group")
            .arg("--ro-bind").arg("/etc/hosts").arg("/etc/hosts")
            .arg("--ro-bind").arg("/etc/resolv.conf").arg("/etc/resolv.conf");

        // Device and proc filesystems
        cmd.arg("--dev").arg("/dev")
            .arg("--proc").arg("/proc");

        // Temporary filesystems
        cmd.arg("--tmpfs").arg("/tmp")
            .arg("--tmpfs").arg("/run");

        // Writable scratch directory
        // The scratch dir on host is bind-mounted as /work inside sandbox
        cmd.arg("--bind")
            .arg(&self.sandbox.scratch_dir)
            .arg("/work");

        // Set working directory to scratch
        cmd.arg("--chdir").arg("/work");

        // Set hostname for isolation
        cmd.arg("--hostname").arg("sandbox");

        // Clear environment and set minimal env
        cmd.arg("--clearenv")
            .arg("--setenv").arg("PATH").arg("/usr/bin:/bin")
            .arg("--setenv").arg("HOME").arg("/work")
            .arg("--setenv").arg("TMPDIR").arg("/tmp");

        // Execute the script via shell
        cmd.arg("/bin/sh").arg("-c").arg(script);

        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
    }
}

/// Read output from a handle, returning empty string if handle is None.
async fn read_handle(handle: Option<tokio::process::ChildStdout>) -> String {
    if let Some(mut handle) = handle {
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        let _ = handle.read_to_end(&mut buf).await;
        String::from_utf8_lossy(&buf).to_string()
    } else {
        String::new()
    }
}

/// Overload for stderr handle type.
async fn read_handle_stderr(handle: Option<tokio::process::ChildStderr>) -> String {
    if let Some(mut handle) = handle {
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        let _ = handle.read_to_end(&mut buf).await;
        String::from_utf8_lossy(&buf).to_string()
    } else {
        String::new()
    }
}
