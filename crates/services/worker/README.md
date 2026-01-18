# Vulcan Worker

Executes individual chain fragments and reports results back to the orchestrator.

## Status

**Implemented** - Basic worker functionality complete.

## Running

```bash
task run-worker
```

Or manually with environment variables:

```bash
ORCHESTRATOR_URL=http://localhost:3002 \
TENANT_ID=<uuid> \
WORKER_GROUP=default \
cargo run -p vulcan-worker
```

## Configuration

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| `ORCHESTRATOR_URL` | Worker orchestrator endpoint | Yes | - |
| `TENANT_ID` | Tenant UUID this worker belongs to | Yes | - |
| `WORKER_GROUP` | Machine group this worker belongs to | No | - |
| `HEARTBEAT_INTERVAL_SECS` | Heartbeat frequency in seconds | No | 10 |
| `POLL_INTERVAL_SECS` | Work polling frequency in seconds | No | 5 |
| `REQUEST_TIMEOUT_SECS` | HTTP request timeout in seconds | No | 30 |
| `SCRIPT_TIMEOUT_SECS` | Script execution timeout in seconds | No | 300 |

## Architecture

### Components

- **Config** (`config.rs`): Environment-based configuration loading
- **Error** (`error.rs`): Error types using thiserror
- **Client** (`client/`): HTTP client for orchestrator API communication
- **Executor** (`executor/`): Script execution with timeout enforcement
- **Worker** (`worker.rs`): State machine with concurrent heartbeat and work loop

### Orchestrator API

The worker communicates with the orchestrator via these endpoints:

- `POST /workers/register` - Register worker
- `POST /workers/heartbeat` - Send heartbeat
- `POST /work/request` - Request work (returns 204 if none available)
- `POST /work/result` - Report execution result

### Retry Logic

The worker implements exponential backoff for:
- Registration failures
- Heartbeat failures
- Work request failures

Backoff starts at 1 second, doubles on each failure, and caps at 60 seconds.

### Graceful Shutdown

The worker handles Ctrl+C for graceful shutdown:
- Stops requesting new work
- Completes current work execution
- Stops heartbeat task

## Implemented Functionality

- Worker registration with orchestrator
- Periodic heartbeats (background task)
- Work polling and execution
- Script execution via `/bin/sh -c`
- stdout/stderr capture
- Exit code reporting
- Timeout enforcement for scripts
- Graceful shutdown (Ctrl+C)
- Exponential backoff retry logic

## Future Improvements

- Container-based execution
- stdout/stderr streaming
- Environment variable injection
- Secret injection
- Resource limit enforcement
- OpenTelemetry integration
