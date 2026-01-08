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
mod repositories;
mod schema;

use uuid::Uuid;

use db::{establish_connection, run_migrations};
use models::chain::{ChainStatus, NewChain};
use models::fragment::{FragmentStatus, NewFragment};
use models::worker::{NewWorker, WorkerStatus};
use repositories::{
    ChainRepository, FragmentRepository, PgChainRepository, PgFragmentRepository,
    PgWorkerRepository, WorkerRepository,
};

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

    // Create a chain using the repository
    let chain_id = Uuid::new_v4();
    let chain = {
        let mut repo = PgChainRepository::new(&mut connection);
        let new_chain = NewChain {
            id: chain_id,
            tenant_id,
            status: ChainStatus::Active,
            attempt: 1,
        };
        repo.create(new_chain).expect("Failed to create chain")
    };

    println!("\nChain created:");
    println!("  ID: {}", chain.id);
    println!("  Status: {:?}", chain.status);
    println!("  Created at: {}", chain.created_at);

    // Create fragments using the repository
    let fragments = {
        let mut repo = PgFragmentRepository::new(&mut connection);
        let new_fragments = vec![
            NewFragment {
                id: Uuid::new_v4(),
                chain_id,
                attempt: 1,
                status: FragmentStatus::Active,
            },
            NewFragment {
                id: Uuid::new_v4(),
                chain_id,
                attempt: 1,
                status: FragmentStatus::Active,
            },
        ];
        repo.create_many(new_fragments)
            .expect("Failed to create fragments")
    };

    println!("\nFragments created:");
    for (i, fragment) in fragments.iter().enumerate() {
        println!("  Fragment {} ID: {}", i + 1, fragment.id);
    }

    // Create a worker using the repository
    let worker = {
        let mut repo = PgWorkerRepository::new(&mut connection);
        let new_worker = NewWorker {
            id: Uuid::new_v4(),
            tenant_id,
            status: WorkerStatus::Active,
            current_chain_id: Some(chain_id),
            previous_chain_id: None,
            next_chain_id: None,
        };
        repo.create(new_worker).expect("Failed to create worker")
    };

    println!("\nWorker created:");
    println!("  ID: {}", worker.id);
    println!("  Status: {:?}", worker.status);
    println!("  Current Chain: {:?}", worker.current_chain_id);

    // Query counts and demonstrate repository usage
    let (worker_count, chain_count, fragment_count, chain_fragments) = {
        let mut worker_repo = PgWorkerRepository::new(&mut connection);
        let wc = worker_repo.count().expect("Failed to count workers");

        // Reborrow for chain repo
        let mut chain_repo = PgChainRepository::new(worker_repo.conn());
        let cc = chain_repo.count().expect("Failed to count chains");

        // Reborrow for fragment repo
        let mut fragment_repo = PgFragmentRepository::new(chain_repo.conn());
        let fc = fragment_repo.count().expect("Failed to count fragments");
        let cf = fragment_repo
            .find_by_chain(chain_id)
            .expect("Failed to find fragments");

        (wc, cc, fc, cf)
    };

    println!("\n--- Database Summary ---");
    println!("  Workers: {worker_count}");
    println!("  Chains: {chain_count}");
    println!("  Fragments: {fragment_count}");
    println!(
        "\nFragments in chain {}: {}",
        chain_id,
        chain_fragments.len()
    );

    println!("\nVulcan CI initialized successfully!");
}
