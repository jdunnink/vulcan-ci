use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::workers;

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::WorkerStatus"]
pub enum WorkerStatus {
    Active,
    Suspended,
    Error,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = workers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Worker {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub status: WorkerStatus,
    pub current_chain_id: Option<Uuid>,
    pub previous_chain_id: Option<Uuid>,
    pub next_chain_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = workers)]
pub struct NewWorker {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub status: WorkerStatus,
    pub current_chain_id: Option<Uuid>,
    pub previous_chain_id: Option<Uuid>,
    pub next_chain_id: Option<Uuid>,
}
