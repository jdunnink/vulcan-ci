use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::chains;

/// Status of a chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ChainStatus"]
pub enum ChainStatus {
    /// Chain is active and processing.
    Active,
    /// Chain is suspended.
    Suspended,
    /// Chain encountered an error.
    Error,
}

/// Type of trigger that initiated the chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::TriggerType"]
pub enum TriggerType {
    /// Triggered by a git tag.
    Tag,
    /// Triggered by a push to a branch.
    Push,
    /// Triggered by a pull request.
    PullRequest,
    /// Triggered by a schedule.
    Schedule,
    /// Manually triggered.
    Manual,
}

/// Represents a chain entity in the database.
/// Field order must match schema column order for Queryable.
#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = chains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chain {
    /// Unique identifier for the chain.
    pub id: Uuid,
    /// Tenant this chain belongs to.
    pub tenant_id: Uuid,
    /// Current status of the chain.
    pub status: ChainStatus,
    /// Number of attempts made.
    pub attempt: i32,
    /// When the chain was created.
    pub created_at: NaiveDateTime,
    /// When the chain was last updated.
    pub updated_at: NaiveDateTime,
    /// Path to the workflow file that defined this chain.
    pub source_file_path: Option<String>,
    /// URL of the repository containing the workflow.
    pub repository_url: Option<String>,
    /// Git commit SHA that triggered this chain.
    pub commit_sha: Option<String>,
    /// Git branch name.
    pub branch: Option<String>,
    /// Type of trigger that initiated this chain.
    pub trigger: Option<TriggerType>,
    /// Reference for the trigger (e.g., tag name, PR number).
    pub trigger_ref: Option<String>,
    /// Default machine/worker group for fragments that don't specify one.
    pub default_machine: Option<String>,
}

/// Data for creating a new chain.
#[derive(Debug, Insertable)]
#[diesel(table_name = chains)]
pub struct NewChain {
    /// Unique identifier for the chain.
    pub id: Uuid,
    /// Tenant this chain belongs to.
    pub tenant_id: Uuid,
    /// Initial status of the chain.
    pub status: ChainStatus,
    /// Initial attempt count.
    pub attempt: i32,
    /// Path to the workflow file.
    pub source_file_path: Option<String>,
    /// URL of the repository.
    pub repository_url: Option<String>,
    /// Git commit SHA.
    pub commit_sha: Option<String>,
    /// Git branch name.
    pub branch: Option<String>,
    /// Type of trigger.
    pub trigger: Option<TriggerType>,
    /// Trigger reference.
    pub trigger_ref: Option<String>,
    /// Default machine/worker group.
    pub default_machine: Option<String>,
}

impl NewChain {
    /// Create a new chain with minimal required fields.
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            status: ChainStatus::Active,
            attempt: 1,
            source_file_path: None,
            repository_url: None,
            commit_sha: None,
            branch: None,
            trigger: None,
            trigger_ref: None,
            default_machine: None,
        }
    }

    /// Set the source file path.
    pub fn with_source(mut self, path: String) -> Self {
        self.source_file_path = Some(path);
        self
    }

    /// Set the repository URL.
    pub fn with_repository(mut self, url: String) -> Self {
        self.repository_url = Some(url);
        self
    }

    /// Set the commit SHA.
    pub fn with_commit(mut self, sha: String) -> Self {
        self.commit_sha = Some(sha);
        self
    }

    /// Set the branch name.
    pub fn with_branch(mut self, branch: String) -> Self {
        self.branch = Some(branch);
        self
    }

    /// Set the trigger information.
    pub fn with_trigger(mut self, trigger_type: TriggerType, trigger_ref: Option<String>) -> Self {
        self.trigger = Some(trigger_type);
        self.trigger_ref = trigger_ref;
        self
    }

    /// Set the default machine/worker group.
    pub fn with_default_machine(mut self, machine: String) -> Self {
        self.default_machine = Some(machine);
        self
    }
}
