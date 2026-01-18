# Vulcan CI

A modern, distributed continuous integration system written in Rust with a declarative KDL-based workflow format.

## Features

- **Declarative Workflows**: Define CI/CD pipelines using KDL (KDL Document Language)
- **Parallel Execution**: Run tasks concurrently with the `parallel` block
- **Reusable Fragments**: Import shared tasks from URLs
- **Conditional Logic**: Execute tasks based on environment variables
- **Multiple Triggers**: Support for push, pull request, tag, schedule, and manual triggers
- **Distributed Workers**: Execute workflows across multiple worker machines
- **PostgreSQL Backend**: Reliable storage for workflows, tasks, and execution state

## Workflow Format

Workflows are defined in `.kdl` files:

```kdl
version "0.1"
triggers "push" "pull_request"

chain {
    machine "default-worker"

    fragment { run "npm install" }

    parallel {
        fragment { run "npm test" }
        fragment { run "npm lint" }
        fragment { run "npm build" }
    }

    fragment {
        from "https://github.com/org/shared/deploy.kdl"
        condition "$BRANCH == 'main'"
    }
}
```

See [PARSER_SPEC.md](crates/libs/chain-parser/PARSER_SPEC.md) for the complete workflow specification.

## Architecture

Vulcan CI is organized as a Rust workspace with libraries and services:

### Libraries

Shared crates used as dependencies by services:

| Crate | Description |
|-------|-------------|
| `vulcan-core` | Shared data models, database schema, repositories |
| `vulcan-chain-parser` | KDL workflow parser and AST types |

### Services

Deployable binaries that make up the system:

| Crate | Description |
|-------|-------------|
| `vulcan-api` | Main API for managing workflows and workers |
| `vulcan-worker` | Executes individual workflow fragments |
| `vulcan-worker-orchestrator` | Coordinates work distribution across workers |
| `vulcan-workflow-trigger-processor` | Processes Git events and triggers workflows |
| `vulcan-chain-parser-api` | HTTP API for parsing and storing workflows |
| `vulcan-chain-parser-cli` | CLI tool for validating workflow files |

## Getting Started

### Prerequisites

- Rust (Edition 2024)
- PostgreSQL 15+
- Docker & Docker Compose (optional, for local database)
- [Task](https://taskfile.dev/) runner

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/your-org/vulcan-ci.git
   cd vulcan-ci
   ```

2. Copy environment configuration:
   ```bash
   cp .env.example .env
   ```

3. Start PostgreSQL:
   ```bash
   task db-up
   ```

4. Run database migrations:
   ```bash
   task db-migrate
   ```

5. Build the project:
   ```bash
   task build
   ```

### Running Services

```bash
task run-api                         # Main API
task run-worker                      # Worker service
task run-worker-orchestrator         # Worker orchestrator
task run-workflow-trigger-processor  # Trigger processor
task run-chain-parser-api            # Parser API
```

### Validating Workflows

Use the CLI to validate workflow files:

```bash
cargo run -p vulcan-chain-parser-cli -- path/to/workflow.kdl
```

## Development

### Common Tasks

```bash
task build    # Debug build
task release  # Release build
task check    # Fast compile check
task fmt      # Format code
task lint     # Run clippy
task test     # Run tests
task ci       # Full CI pipeline
```

### Database Tasks

```bash
task db-up      # Start PostgreSQL container
task db-down    # Stop PostgreSQL container
task db-reset   # Reset database
task db-migrate # Run migrations
task db-psql    # Open psql shell
```

## Project Structure

```
vulcan-ci/
├── crates/
│   ├── libs/                        # Shared libraries
│   │   ├── core/                    # Data models, schema, repositories
│   │   └── chain-parser/            # KDL workflow parser
│   │
│   └── services/                    # Deployable binaries
│       ├── api/                     # Main API service
│       ├── worker/                  # Fragment execution service
│       ├── worker-orchestrator/     # Work distribution service
│       ├── workflow-trigger-processor/
│       ├── chain-parser-api/        # Parser HTTP service
│       └── chain-parser-cli/        # Parser CLI tool
│
├── migrations/                      # Diesel database migrations
├── Cargo.toml                       # Workspace configuration
├── Dockerfile                       # Multi-stage build for services
├── docker-compose.yml               # PostgreSQL + services
└── Taskfile.yml                     # Development tasks
```

## Docker

Run the full stack with Docker Compose:

```bash
# Start all services
docker compose up -d

# Run database migrations
task db-migrate

# Check status
docker compose ps

# View logs
docker compose logs -f worker-orchestrator

# Stop services
docker compose down
```

## Status

Vulcan CI is in active development. The core execution engine is complete: libraries (data models, workflow parser), parser tooling, worker orchestrator, and worker execution service with security sandboxing. The trigger processor is next.

| Component | Status |
|-----------|--------|
| Libraries | Complete |
| Parser CLI & API | Complete |
| Worker Orchestrator | Complete |
| Worker Service | Complete |
| Trigger Processor | Planned |

See [ROADMAP.md](ROADMAP.md) for detailed implementation status and planned features.

## License

[Add license information here]
