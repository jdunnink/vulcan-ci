# Vulcan Workflow Trigger Processor

Processes incoming events and triggers workflow execution when conditions match.

## Status

**Scaffold** - Placeholder implementation only.

## Running

```bash
task run-workflow-trigger-processor
```

## Configuration

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `PORT` | HTTP server port for webhooks | No (default: 3003) |
| `WEBHOOK_SECRET` | Secret for webhook signature verification | Yes |

## Planned Functionality

- Webhook receiver for Git events (push, pull request, tag)
- Trigger matching against workflow definitions
- Chain creation when triggers match
- Webhook signature verification (GitHub, GitLab, etc.)
- Multi-provider support (GitHub, GitLab, Gitea, Bitbucket)
- Schedule trigger support (cron-based)
- Manual trigger API endpoint
- Event deduplication and rate limiting
- Native OpenTelemetry support for observability
