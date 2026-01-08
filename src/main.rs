//! Vulcan CI - A distributed CI/CD work execution engine
//!
//! This system manages CI/CD workflows through a chain-based execution model:
//! - **Workers**: Execution agents that process chains of work
//! - **Chains**: Collections of fragments that need to be executed together
//! - **Fragments**: Atomic units of work
//!
//! The system supports multi-tenancy, retry logic, and distributed execution.

mod db;
mod models;
mod schema;

use diesel::prelude::*;
use uuid::Uuid;

use db::{establish_connection, run_migrations};
use models::chain::{ChainStatus, NewChain};
use models::fragment::{FragmentStatus, NewFragment};
use models::worker::{NewWorker, WorkerStatus};
use schema::{chains, fragments, workers};

fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();

    println!("Vulcan CI - Initializing...\n");

    // Establish database connection
    let mut connection = establish_connection();
    println!("Connected to database");

    // Run migrations on startup
    run_migrations(&mut connection);
    println!("Migrations completed\n");

    // Initialize tenant ID (shared across all entities)
    let tenant_id = Uuid::new_v4();
    println!("Tenant ID: {tenant_id}");

    // Create a chain
    let chain_id = Uuid::new_v4();
    let new_chain = NewChain {
        id: chain_id,
        tenant_id,
        status: ChainStatus::Active,
        attempt: 1,
    };

    diesel::insert_into(chains::table)
        .values(&new_chain)
        .execute(&mut connection)
        .expect("Error inserting chain");

    println!("\nChain created:");
    println!("  ID: {chain_id}");
    println!("  Status: {:?}", ChainStatus::Active);

    // Create fragments for the chain
    let fragment1_id = Uuid::new_v4();
    let new_fragment1 = NewFragment {
        id: fragment1_id,
        chain_id,
        attempt: 1,
        status: FragmentStatus::Active,
    };

    let fragment2_id = Uuid::new_v4();
    let new_fragment2 = NewFragment {
        id: fragment2_id,
        chain_id,
        attempt: 1,
        status: FragmentStatus::Active,
    };

    diesel::insert_into(fragments::table)
        .values(&[new_fragment1, new_fragment2])
        .execute(&mut connection)
        .expect("Error inserting fragments");

    println!("\nFragments created:");
    println!("  Fragment 1 ID: {fragment1_id}");
    println!("  Fragment 2 ID: {fragment2_id}");

    // Create a worker to process the chain
    let worker_id = Uuid::new_v4();
    let new_worker = NewWorker {
        id: worker_id,
        tenant_id,
        status: WorkerStatus::Active,
        current_chain_id: Some(chain_id),
        previous_chain_id: None,
        next_chain_id: None,
    };

    diesel::insert_into(workers::table)
        .values(&new_worker)
        .execute(&mut connection)
        .expect("Error inserting worker");

    println!("\nWorker created:");
    println!("  ID: {worker_id}");
    println!("  Status: {:?}", WorkerStatus::Active);
    println!("  Current Chain: {chain_id}");

    // Query and display the counts
    let worker_count: i64 = workers::table
        .count()
        .get_result(&mut connection)
        .expect("Error counting workers");

    let chain_count: i64 = chains::table
        .count()
        .get_result(&mut connection)
        .expect("Error counting chains");

    let fragment_count: i64 = fragments::table
        .count()
        .get_result(&mut connection)
        .expect("Error counting fragments");

    println!("\n--- Database Summary ---");
    println!("  Workers: {worker_count}");
    println!("  Chains: {chain_count}");
    println!("  Fragments: {fragment_count}");

    println!("\nVulcan CI initialized successfully!");
}
