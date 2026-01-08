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

/// Represents a worker entity in the database.
#[derive(Debug, Queryable, Selectable, Identifiable)]
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
}
