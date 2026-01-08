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

/// Represents a chain entity in the database.
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
}
