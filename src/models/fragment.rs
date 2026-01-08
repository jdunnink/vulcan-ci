use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::fragments;

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::FragmentStatus"]
pub enum FragmentStatus {
    Active,
    Suspended,
    Error,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = fragments)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Fragment {
    pub id: Uuid,
    pub chain_id: Uuid,
    pub attempt: i32,
    pub status: FragmentStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = fragments)]
pub struct NewFragment {
    pub id: Uuid,
    pub chain_id: Uuid,
    pub attempt: i32,
    pub status: FragmentStatus,
}
