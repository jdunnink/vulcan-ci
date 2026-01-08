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
}

/// Represents a fragment entity in the database.
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
}

/// Data for creating a new fragment.
#[derive(Debug, Insertable)]
#[diesel(table_name = fragments)]
pub struct NewFragment {
    /// Unique identifier for the fragment.
    pub id: Uuid,
    /// Chain this fragment belongs to.
    pub chain_id: Uuid,
    /// Initial attempt count.
    pub attempt: i32,
    /// Initial status of the fragment.
    pub status: FragmentStatus,
}
