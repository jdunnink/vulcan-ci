//! Output types for script execution.

/// Output from script execution.
#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    /// Standard output from the script.
    pub stdout: String,
    /// Standard error from the script.
    pub stderr: String,
    /// Exit code from the script.
    pub exit_code: i32,
    /// Whether the execution was successful (exit code 0).
    pub success: bool,
    /// Whether the script timed out.
    pub timed_out: bool,
}

impl ExecutionOutput {
    /// Create a new execution output.
    #[must_use]
    pub fn new(stdout: String, stderr: String, exit_code: i32) -> Self {
        Self {
            stdout,
            stderr,
            exit_code,
            success: exit_code == 0,
            timed_out: false,
        }
    }

    /// Create a timeout output.
    #[must_use]
    pub fn timeout(stdout: String, stderr: String) -> Self {
        Self {
            stdout,
            stderr,
            exit_code: -1,
            success: false,
            timed_out: true,
        }
    }

    /// Get an error message if the execution failed.
    #[must_use]
    pub fn error_message(&self) -> Option<String> {
        if self.success {
            return None;
        }

        if self.timed_out {
            return Some("Script execution timed out".to_string());
        }

        if self.stderr.is_empty() {
            Some(format!("Script exited with code {}", self.exit_code))
        } else {
            Some(self.stderr.clone())
        }
    }
}
