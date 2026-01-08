use chrono::NaiveDateTime;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::chains;

#[derive(Debug, Clone, Copy, PartialEq, Eq, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::ChainStatus"]
pub enum ChainStatus {
    Active,
    Suspended,
    Error,
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = chains)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Chain {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub status: ChainStatus,
    pub attempt: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = chains)]
pub struct NewChain {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub status: ChainStatus,
    pub attempt: i32,
}
