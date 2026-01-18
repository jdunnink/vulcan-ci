# Vulcan Worker

Executes individual chain fragments and reports results back to the orchestrator.

## Status

**Scaffold** - Placeholder implementation only.

## Running

```bash
task run-worker
```

## Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `ORCHESTRATOR_URL` | Worker orchestrator endpoint | Yes |
| `WORKER_GROUP` | Machine group this worker belongs to | No |

## Planned Functionality

- Fragment script execution engine (shell, container)
- stdout/stderr capture and streaming
- Status reporting to orchestrator (progress, completion, failure)
- Environment variable and secret injection
- Timeout and resource limit enforcement
- Graceful shutdown with work handoff
- Native OpenTelemetry support for observability
