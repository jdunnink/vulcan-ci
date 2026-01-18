# Vulcan Chain Parser API

HTTP service for parsing KDL workflow files and storing them in PostgreSQL.

## Status

**Complete** - Fully functional with integration tests.

## Running

```bash
task run-chain-parser-api
```

The service starts on port 3001 by default.

## Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `PORT` | HTTP server port | No (default: 3001) |
| `RUST_LOG` | Log level filter | No (default: info) |

## API Endpoints

### Health Check

```
GET /health
```

Returns `200 OK` if the service is running.

### Parse Workflow

```
POST /parse
Content-Type: application/json
```

Parses a KDL workflow and stores the chain and fragments in the database.

**Request Body:**

```json
{
  "content": "version \"0.1\"\ntriggers \"push\"\n\nchain {\n  fragment { run \"echo hello\" }\n}",
  "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
  "source_file_path": ".vulcan/ci.kdl",
  "repository_url": "https://github.com/org/repo",
  "commit_sha": "abc123",
  "branch": "main",
  "trigger": "push",
  "trigger_ref": "refs/heads/main"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `content` | string | Yes | KDL workflow content |
| `tenant_id` | UUID | Yes | Tenant identifier |
| `source_file_path` | string | No | Path to the workflow file |
| `repository_url` | string | No | Repository URL |
| `commit_sha` | string | No | Git commit SHA |
| `branch` | string | No | Git branch name |
| `trigger` | string | No | Trigger type (push, pull_request, tag, schedule, manual) |
| `trigger_ref` | string | No | Trigger reference (e.g., refs/heads/main) |

**Response:**

```json
{
  "chain_id": "550e8400-e29b-41d4-a716-446655440001",
  "fragment_count": 3,
  "message": "Workflow parsed and stored successfully"
}
```

**Error Response:**

```json
{
  "error": "parse_error",
  "message": "Invalid KDL syntax at line 5"
}
```

## Notes

- Imports (`from` directive) are disabled in API mode
- Database migrations are run automatically on startup
- The service validates trigger types against workflow definitions when a trigger is provided
