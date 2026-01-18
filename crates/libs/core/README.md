# Vulcan Core

Shared data models, database schema, and repository implementations used across all Vulcan services.

## Status

**Complete** - Fully functional with all models and repositories implemented.

## Usage

Add to your `Cargo.toml` (from a service crate):

```toml
[dependencies]
vulcan-core = { path = "../../libs/core" }
```

Or use the workspace dependency:

```toml
[dependencies]
vulcan-core.workspace = true
```

## Modules

### Models

Domain entities with Diesel mappings:

- **Chain** - Workflow execution chain with status tracking
- **Fragment** - Individual workflow step (inline script or parallel group)
- **Worker** - Execution worker with health and assignment tracking

### Repositories

Data access layer with trait abstractions:

- **ChainRepository** / `PgChainRepository` - CRUD operations for chains
- **FragmentRepository** / `PgFragmentRepository` - CRUD and bulk operations for fragments
- **WorkerRepository** / `PgWorkerRepository` - CRUD operations for workers

### Database

Connection and migration utilities:

- `establish_connection()` - Create a PostgreSQL connection from `DATABASE_URL`
- `run_migrations()` - Apply pending Diesel migrations

## Example

```rust
use vulcan_core::{
    establish_connection, run_migrations,
    Chain, ChainStatus, NewChain,
    ChainRepository, PgChainRepository,
};
use uuid::Uuid;

// Connect and migrate
let mut conn = establish_connection();
run_migrations(&mut conn);

// Create a chain
let new_chain = NewChain::builder()
    .tenant_id(Uuid::new_v4())
    .status(ChainStatus::Active)
    .build();

let mut repo = PgChainRepository::new(&mut conn);
let chain = repo.create(new_chain).unwrap();
```

## Schema

The database schema is managed via Diesel migrations in `/migrations`. Key tables:

- `chains` - Workflow executions
- `fragments` - Individual tasks within chains
- `workers` - Execution workers

## Features

- `migrations` - Include migration runner (enabled by default for services)
