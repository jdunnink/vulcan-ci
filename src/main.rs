//! Vulcan CI - A distributed CI/CD work execution engine
//!
//! This system manages CI/CD workflows through a chain-based execution model:
//! - **Workers**: Execution agents that process chains of work
//! - **Chains**: Collections of fragments that need to be executed together
//! - **Fragments**: Atomic units of work
//!
//! The system supports multi-tenancy, retry logic, and distributed execution.

mod models;

use models::chain::{Chain, ChainStatus};
use models::fragment::{Fragment, FragmentStatus};
use models::worker::{Worker, WorkerStatus};
use uuid::Uuid;

fn main() {
    println!("Vulcan CI - Initializing models...\n");

    // Initialize tenant ID (shared across all entities)
    let tenant_id = Uuid::new_v4();
    println!("Tenant ID: {tenant_id}");

    // Create some fragments
    let fragment1 = Fragment {
        id: Uuid::new_v4(),
        chain_id: Uuid::new_v4(), // Will be updated when chain is created
        attempt: 1,
        status: FragmentStatus::Active,
    };
    println!("\nFragment 1:");
    println!("  ID: {}", fragment1.id);
    println!("  Status: {:?}", fragment1.status);
    println!("  Attempt: {}", fragment1.attempt);

    let fragment2 = Fragment {
        id: Uuid::new_v4(),
        chain_id: fragment1.chain_id, // Same chain
        attempt: 1,
        status: FragmentStatus::Active,
    };
    println!("\nFragment 2:");
    println!("  ID: {}", fragment2.id);
    println!("  Status: {:?}", fragment2.status);

    // Create a chain containing the fragments
    let chain = Chain {
        id: fragment1.chain_id,
        tenant_id,
        status: ChainStatus::Active,
        attempt: 1,
        fragments: vec![fragment1.id, fragment2.id],
    };
    println!("\nChain:");
    println!("  ID: {}", chain.id);
    println!("  Tenant ID: {}", chain.tenant_id);
    println!("  Status: {:?}", chain.status);
    println!("  Fragments: {} total", chain.fragments.len());

    // Create a worker to process the chain
    let worker = Worker {
        id: Uuid::new_v4(),
        tenant_id,
        status: WorkerStatus::Active,
        current_chain_id: chain.id,
        previous_chain_id: Uuid::nil(), // No previous chain
        next_chain_id: Uuid::nil(),     // No next chain
    };
    println!("\nWorker:");
    println!("  ID: {}", worker.id);
    println!("  Tenant ID: {}", worker.tenant_id);
    println!("  Status: {:?}", worker.status);
    println!("  Current Chain: {}", worker.current_chain_id);

    println!("\nModels initialized successfully!");
}
