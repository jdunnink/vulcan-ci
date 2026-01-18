# Vulcan Worker Orchestrator

Manages worker lifecycle and coordinates fragment execution across the worker pool.

## Status

**Scaffold** - Placeholder implementation only.

## Running

```bash
task run-worker-orchestrator
```

## Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `PORT` | HTTP server port | No (default: 3002) |

## Planned Functionality

- Worker registration and heartbeat monitoring
- Work queue management and prioritization
- Fragment-to-worker assignment based on machine groups
- Parallel vs sequential execution coordination
- Failure detection and retry logic
- Chain/fragment status updates in database
- Dead worker detection and work reassignment
- Native OpenTelemetry support for observability
