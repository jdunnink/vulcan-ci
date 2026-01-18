use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::fragments;

/// Status of a fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::FragmentStatus"]
pub enum FragmentStatus {
    /// Fragment is active and processing.
    Active,
    /// Fragment is suspended.
    Suspended,
    /// Fragment encountered an error.
    Error,
    /// Fragment is waiting to be executed.
    Pending,
    /// Fragment is currently being executed.
    Running,
    /// Fragment execution completed successfully.
    Completed,
    /// Fragment execution failed.
    Failed,
}

impl FragmentStatus {
    /// Returns true if the fragment is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, FragmentStatus::Completed | FragmentStatus::Failed)
    }

    /// Returns true if the fragment is ready to be scheduled.
    pub fn is_pending(&self) -> bool {
        matches!(self, FragmentStatus::Pending)
    }

    /// Returns true if the fragment completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(self, FragmentStatus::Completed)
    }
}

/// Type of a fragment (stored in DB).
/// Note: Import is not stored - imports are resolved at parse time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::FragmentType"]
pub enum FragmentType {
    /// Fragment contains inline script to execute.
    Inline,
    /// Fragment is a group container (for parallel execution).
    Group,
}

/// Represents a fragment entity in the database.
///
/// Fragments form a tree structure where:
/// - Top-level fragments have `parent_fragment_id = None`
/// - Children are ordered by `sequence` within their parent
/// - If `is_parallel = true`, children execute concurrently
/// - If `condition` is set, fragment only executes if condition evaluates to true
///
/// Note: Import fragments are resolved at parse time and not stored.
/// The `source_url` field tracks where a fragment was imported from.
#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = fragments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Fragment {
    /// Unique identifier for the fragment.
    pub id: Uuid,
    /// Chain this fragment belongs to.
    pub chain_id: Uuid,
    /// Number of attempts made.
    pub attempt: i32,
    /// Current status of the fragment.
    pub status: FragmentStatus,
    /// When the fragment was created.
    pub created_at: NaiveDateTime,
    /// When the fragment was last updated.
    pub updated_at: NaiveDateTime,
    /// Parent fragment (None if top-level).
    pub parent_fragment_id: Option<Uuid>,
    /// Execution order within siblings.
    pub sequence: i32,
    /// Type of fragment (inline or group).
    #[diesel(column_name = type_)]
    pub fragment_type: FragmentType,
    /// Script to execute (for inline fragments).
    pub run_script: Option<String>,
    /// Worker group/machine to execute on.
    pub machine: Option<String>,
    /// If true, children execute in parallel; if false, sequentially.
    pub is_parallel: bool,
    /// Condition expression; fragment skipped if evaluates to false.
    pub condition: Option<String>,
    /// URL this fragment was imported from (None if defined inline).
    pub source_url: Option<String>,
    /// Worker currently assigned to execute this fragment.
    pub assigned_worker_id: Option<Uuid>,
    /// When execution started.
    pub started_at: Option<NaiveDateTime>,
    /// When execution completed.
    pub completed_at: Option<NaiveDateTime>,
    /// Exit code from execution (0 = success).
    pub exit_code: Option<i32>,
    /// Error message if execution failed.
    pub error_message: Option<String>,
}

/// Data for creating a new fragment.
#[derive(Debug, Insertable)]
#[diesel(table_name = fragments)]
pub struct NewFragment {
    /// Unique identifier for the fragment.
    pub id: Uuid,
    /// Chain this fragment belongs to.
    pub chain_id: Uuid,
    /// Parent fragment ID.
    pub parent_fragment_id: Option<Uuid>,
    /// Execution order within siblings.
    pub sequence: i32,
    /// Type of fragment.
    #[diesel(column_name = type_)]
    pub fragment_type: FragmentType,
    /// Script to execute (for inline fragments).
    pub run_script: Option<String>,
    /// Worker group/machine to execute on.
    pub machine: Option<String>,
    /// If true, children execute in parallel.
    pub is_parallel: bool,
    /// Condition expression.
    pub condition: Option<String>,
    /// URL this fragment was imported from.
    pub source_url: Option<String>,
    /// Initial attempt count.
    pub attempt: i32,
    /// Initial status of the fragment.
    pub status: FragmentStatus,
}

impl NewFragment {
    /// Create a new inline fragment.
    pub fn inline(chain_id: Uuid, sequence: i32, run_script: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            chain_id,
            parent_fragment_id: None,
            sequence,
            fragment_type: FragmentType::Inline,
            run_script: Some(run_script),
            machine: None,
            is_parallel: false,
            condition: None,
            source_url: None,
            attempt: 1,
            status: FragmentStatus::Active,
        }
    }

    /// Create a new parallel group fragment.
    pub fn parallel_group(chain_id: Uuid, sequence: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            chain_id,
            parent_fragment_id: None,
            sequence,
            fragment_type: FragmentType::Group,
            run_script: None,
            machine: None,
            is_parallel: true,
            condition: None,
            source_url: None,
            attempt: 1,
            status: FragmentStatus::Active,
        }
    }

    /// Set the parent fragment ID.
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_fragment_id = Some(parent_id);
        self
    }

    /// Set the machine/worker group.
    pub fn with_machine(mut self, machine: String) -> Self {
        self.machine = Some(machine);
        self
    }

    /// Set a condition for execution.
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Set the source URL (for imported fragments).
    pub fn with_source_url(mut self, url: String) -> Self {
        self.source_url = Some(url);
        self
    }
}
