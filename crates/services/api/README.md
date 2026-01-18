# Vulcan API

Main HTTP API service for managing workflows, chains, and workers.

## Status

**Scaffold** - Currently only runs database migrations on startup.

## Running

```bash
task run-api
```

## Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |

## Planned Functionality

- Chain management endpoints (list, get, cancel, retry)
- Worker management endpoints (register, status, health)
- Execution status and logs retrieval
- Administrative endpoints (pause/resume system, configuration)
- Native OpenTelemetry support for observability
