// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "chain_status"))]
    pub struct ChainStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "fragment_status"))]
    pub struct FragmentStatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "fragment_type"))]
    pub struct FragmentType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "trigger_type"))]
    pub struct TriggerType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "worker_status"))]
    pub struct WorkerStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ChainStatus;
    use super::sql_types::TriggerType;

    chains (id) {
        id -> Uuid,
        tenant_id -> Uuid,
        status -> ChainStatus,
        attempt -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        source_file_path -> Nullable<Text>,
        repository_url -> Nullable<Text>,
        commit_sha -> Nullable<Text>,
        branch -> Nullable<Text>,
        trigger -> Nullable<TriggerType>,
        trigger_ref -> Nullable<Text>,
        default_machine -> Nullable<Text>,
        started_at -> Nullable<Timestamp>,
        completed_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FragmentStatus;
    use super::sql_types::FragmentType;

    fragments (id) {
        id -> Uuid,
        chain_id -> Uuid,
        attempt -> Int4,
        status -> FragmentStatus,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        parent_fragment_id -> Nullable<Uuid>,
        sequence -> Int4,
        #[sql_name = "type"]
        type_ -> FragmentType,
        run_script -> Nullable<Text>,
        machine -> Nullable<Text>,
        is_parallel -> Bool,
        condition -> Nullable<Text>,
        source_url -> Nullable<Text>,
        assigned_worker_id -> Nullable<Uuid>,
        started_at -> Nullable<Timestamp>,
        completed_at -> Nullable<Timestamp>,
        exit_code -> Nullable<Int4>,
        error_message -> Nullable<Text>,
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
        last_heartbeat_at -> Nullable<Timestamp>,
        machine_group -> Nullable<Text>,
        current_fragment_id -> Nullable<Uuid>,
    }
}

diesel::joinable!(fragments -> chains (chain_id));

diesel::allow_tables_to_appear_in_same_query!(chains, fragments, workers,);
