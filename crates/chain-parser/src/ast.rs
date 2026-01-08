//! Abstract Syntax Tree types for parsed KDL workflow definitions.
//!
//! These types represent the parsed structure before conversion to database models.
//! Import fragments are expanded during parsing, so only `Inline` and `Group` remain.

use uuid::Uuid;

/// A parsed workflow chain ready for database storage.
#[derive(Debug, Clone)]
pub struct ParsedChain {
    /// Unique identifier for the chain.
    pub id: Uuid,
    /// Event types that trigger this workflow.
    pub triggers: Vec<String>,
    /// Default machine/worker group for fragments.
    pub default_machine: String,
    /// Flattened list of fragments (imports resolved).
    pub fragments: Vec<ParsedFragment>,
}

/// A parsed fragment (either inline script or group container).
#[derive(Debug, Clone)]
pub struct ParsedFragment {
    /// Unique identifier for this fragment.
    pub id: Uuid,
    /// Parent fragment ID (for children of parallel groups).
    pub parent_id: Option<Uuid>,
    /// Execution order within siblings.
    pub sequence: i32,
    /// Type of fragment.
    pub fragment_type: ParsedFragmentType,
    /// Script to execute (for inline fragments).
    pub run_script: Option<String>,
    /// Worker group/machine override.
    pub machine: Option<String>,
    /// Whether children execute in parallel (for group fragments).
    pub is_parallel: bool,
    /// Condition expression for conditional execution.
    pub condition: Option<String>,
    /// URL this fragment was imported from (None if defined inline).
    pub source_url: Option<String>,
}

/// Type of fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsedFragmentType {
    /// Fragment contains inline script to execute.
    Inline,
    /// Fragment is a group container (for parallel execution).
    Group,
}

impl ParsedFragment {
    /// Create a new inline fragment.
    #[must_use]
    pub fn inline(sequence: i32, run_script: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            sequence,
            fragment_type: ParsedFragmentType::Inline,
            run_script: Some(run_script),
            machine: None,
            is_parallel: false,
            condition: None,
            source_url: None,
        }
    }

    /// Create a new parallel group fragment.
    #[must_use]
    pub fn parallel_group(sequence: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent_id: None,
            sequence,
            fragment_type: ParsedFragmentType::Group,
            run_script: None,
            machine: None,
            is_parallel: true,
            condition: None,
            source_url: None,
        }
    }

    /// Set the parent fragment ID.
    #[must_use]
    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set the machine/worker group.
    #[must_use]
    pub fn with_machine(mut self, machine: String) -> Self {
        self.machine = Some(machine);
        self
    }

    /// Set a condition for execution.
    #[must_use]
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Set the source URL (for imported fragments).
    #[must_use]
    pub fn with_source_url(mut self, url: String) -> Self {
        self.source_url = Some(url);
        self
    }
}
