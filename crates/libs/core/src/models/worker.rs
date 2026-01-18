use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::workers;

/// Status of a worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::WorkerStatus"]
pub enum WorkerStatus {
    /// Worker is active and available.
    Active,
    /// Worker is suspended.
    Suspended,
    /// Worker encountered an error.
    Error,
}

impl WorkerStatus {
    /// Returns true if the worker is available to accept work.
    pub fn is_available(&self) -> bool {
        matches!(self, WorkerStatus::Active)
    }
}

/// Represents a worker entity in the database.
#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = workers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Worker {
    /// Unique identifier for the worker.
    pub id: Uuid,
    /// Tenant this worker belongs to.
    pub tenant_id: Uuid,
    /// Current status of the worker.
    pub status: WorkerStatus,
    /// Currently assigned chain, if any.
    pub current_chain_id: Option<Uuid>,
    /// Previously assigned chain, if any.
    pub previous_chain_id: Option<Uuid>,
    /// Next chain to be assigned, if any.
    pub next_chain_id: Option<Uuid>,
    /// When the worker was created.
    pub created_at: NaiveDateTime,
    /// When the worker was last updated.
    pub updated_at: NaiveDateTime,
    /// When the worker last sent a heartbeat.
    pub last_heartbeat_at: Option<NaiveDateTime>,
    /// Machine group this worker belongs to.
    pub machine_group: Option<String>,
    /// Currently assigned fragment, if any.
    pub current_fragment_id: Option<Uuid>,
}

/// Data for creating a new worker.
#[derive(Debug, Insertable)]
#[diesel(table_name = workers)]
pub struct NewWorker {
    /// Unique identifier for the worker.
    pub id: Uuid,
    /// Tenant this worker belongs to.
    pub tenant_id: Uuid,
    /// Initial status of the worker.
    pub status: WorkerStatus,
    /// Initially assigned chain, if any.
    pub current_chain_id: Option<Uuid>,
    /// Previous chain assignment, if any.
    pub previous_chain_id: Option<Uuid>,
    /// Next chain to be assigned, if any.
    pub next_chain_id: Option<Uuid>,
    /// Initial heartbeat timestamp.
    pub last_heartbeat_at: Option<NaiveDateTime>,
    /// Machine group this worker belongs to.
    pub machine_group: Option<String>,
    /// Initially assigned fragment, if any.
    pub current_fragment_id: Option<Uuid>,
}

impl NewWorker {
    /// Create a new worker with minimal required fields.
    pub fn new(tenant_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id,
            status: WorkerStatus::Active,
            current_chain_id: None,
            previous_chain_id: None,
            next_chain_id: None,
            last_heartbeat_at: None,
            machine_group: None,
            current_fragment_id: None,
        }
    }

    /// Set the machine group for this worker.
    pub fn with_machine_group(mut self, machine_group: String) -> Self {
        self.machine_group = Some(machine_group);
        self
    }

    /// Set the initial heartbeat timestamp.
    pub fn with_heartbeat(mut self, heartbeat_at: NaiveDateTime) -> Self {
        self.last_heartbeat_at = Some(heartbeat_at);
        self
    }
}
