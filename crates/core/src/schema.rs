// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "chain_status"))]
    pub struct ChainStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "fragment_status"))]
    pub struct FragmentStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "worker_status"))]
    pub struct WorkerStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ChainStatus;

    chains (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        status -> ChainStatus,
        attempt -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FragmentStatus;

    fragments (id) {
        id -> Uuid,
        chain_id -> Uuid,
        attempt -> Int4,
        status -> FragmentStatus,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::WorkerStatus;

    workers (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        status -> WorkerStatus,
        current_chain_id -> Nullable<Uuid>,
        previous_chain_id -> Nullable<Uuid>,
        next_chain_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::joinable!(fragments -> chains (chain_id));

diesel::allow_tables_to_appear_in_same_query!(chains, fragments, workers,);
