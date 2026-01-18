# Vulcan CI Roadmap

This document outlines the development roadmap for Vulcan CI, organized into phases that align with our go-to-market strategy.

## Current Status

### Complete

| Component | Description |
|-----------|-------------|
| `vulcan-core` | Data models, repositories, PostgreSQL schema |
| `vulcan-chain-parser` | KDL workflow parser with import resolution |
| `vulcan-chain-parser-api` | HTTP API for parsing and storing workflows |
| `vulcan-chain-parser-cli` | CLI tool for workflow validation |

### In Progress

| Component | Status |
|-----------|--------|
| `vulcan-api` | Scaffold only |
| `vulcan-worker` | Scaffold only |
| `vulcan-worker-orchestrator` | **MVP Complete** - Pull-based work distribution |
| `vulcan-workflow-trigger-processor` | Scaffold only |

---

## Phase 1: Core Execution Engine

**Goal:** Achieve a functional self-hosted CI system capable of executing workflows end-to-end.

### 1.1 Worker Orchestrator

The central coordinator for workflow execution.

- [x] Worker registration and heartbeat monitoring
- [ ] Work queue management and prioritization
- [x] Fragment-to-worker assignment based on machine groups
- [x] Parallel vs sequential execution coordination
- [x] Failure detection and retry logic
- [x] Chain/fragment status updates in database
- [x] Dead worker detection and work reassignment

**Implemented Features:**
- Pull-based communication model (workers poll for work)
- HTTP API endpoints: `/workers/register`, `/workers/heartbeat`, `/work/request`, `/work/result`
- Background health monitor for detecting dead workers
- Automatic fragment retry on worker failure (configurable max attempts)
- Sequential/parallel scheduling based on fragment tree structure
- Automatic chain completion when all fragments finish
- Docker support with multi-stage build

### 1.2 Worker Service

Executes individual workflow fragments.

- [ ] Fragment script execution engine (shell-based)
- [ ] stdout/stderr capture and streaming
- [ ] Status reporting to orchestrator
- [ ] Environment variable injection
- [ ] Timeout enforcement
- [ ] Resource limit support
- [ ] Graceful shutdown with work handoff

### 1.3 Workflow Trigger Processor

Ingests events and initiates workflow execution.

- [ ] Webhook receiver for Git events
- [ ] Trigger matching against workflow definitions
- [ ] Chain creation from matched triggers
- [ ] Webhook signature verification (GitHub)
- [ ] Support for push, pull request, and tag events
- [ ] Manual trigger API endpoint

### 1.4 Main API

Management interface for workflows and workers.

- [ ] Chain management endpoints (list, get, cancel)
- [ ] Worker management endpoints (list, status)
- [ ] Execution logs retrieval
- [ ] Health check endpoints

### 1.5 Observability

Native OpenTelemetry support across all services.

- [ ] Tracing with span context propagation
- [ ] Metrics exposition (counters, histograms, gauges)
- [ ] Structured logging with trace correlation
- [ ] OTLP exporter for OpenTelemetry Collector

---

## Phase 2: Production Readiness

**Goal:** Enterprise-grade reliability and initial compliance features.

### 2.1 Multi-Provider Support

Expand trigger processor to support multiple Git providers.

- [ ] GitLab webhook support
- [ ] Gitea webhook support
- [ ] Bitbucket webhook support
- [ ] Generic webhook interface for custom integrations

### 2.2 Secrets Management

Secure handling of sensitive configuration.

- [ ] Encrypted secrets storage
- [ ] Per-repository secret scoping
- [ ] Secret injection into fragment execution
- [ ] Integration with external secret stores (Vault, etc.)

### 2.3 Audit Logging

Comprehensive activity tracking for compliance.

- [ ] Authentication/authorization events
- [ ] Workflow execution history
- [ ] Configuration changes
- [ ] API access logs
- [ ] Immutable audit log storage

### 2.4 Container Execution

Support for containerized workflow execution.

- [ ] Docker-based fragment execution
- [ ] Custom container image support
- [ ] Container resource limits
- [ ] Image pull policies and caching

### 2.5 Scheduled Triggers

Cron-based workflow execution.

- [ ] Cron expression parsing
- [ ] Schedule management API
- [ ] Timezone support
- [ ] Missed schedule handling

---

## Phase 3: Enterprise Features

**Goal:** Features required for enterprise adoption and compliance certification.

### 3.1 Authentication & Authorization

Enterprise identity management.

- [ ] SAML/SSO integration
- [ ] OIDC support
- [ ] Role-based access control (RBAC)
- [ ] Team/organization hierarchy
- [ ] API token management

### 3.2 Multi-Tenancy

Secure isolation for multiple organizations.

- [ ] Tenant isolation guarantees
- [ ] Resource quotas per tenant
- [ ] Tenant-specific configuration
- [ ] Cross-tenant security boundaries

### 3.3 Advanced Compliance

Regulatory compliance support.

- [ ] GDPR data handling controls
- [ ] Data retention policies
- [ ] Right to deletion support
- [ ] NIS2 security controls
- [ ] Compliance reporting

### 3.4 High Availability

Production-grade reliability.

- [ ] Orchestrator clustering
- [ ] Database replication support
- [ ] Graceful degradation
- [ ] Disaster recovery procedures

### 3.5 Advanced Workflow Features

Power-user capabilities.

- [ ] Matrix builds
- [ ] Workflow dependencies (fan-in/fan-out)
- [ ] Approval gates
- [ ] Environment promotions
- [ ] Artifact storage and retrieval

---

## Phase 4: Scale & Ecosystem

**Goal:** Platform ecosystem and advanced integrations.

### 4.1 Plugin System

Extensibility framework.

- [ ] Plugin API specification
- [ ] Community plugin registry
- [ ] Built-in plugin marketplace

### 4.2 Advanced Analytics

Insights and optimization.

- [ ] Build time analytics
- [ ] Failure pattern detection
- [ ] Resource optimization recommendations
- [ ] Cost attribution

### 4.3 Self-Service Administration

Reduced operational overhead.

- [ ] Web-based admin console
- [ ] Self-service onboarding
- [ ] Usage dashboards
- [ ] Billing integration (SaaS)

---

## Contributing

See individual crate READMEs for implementation details:

- [vulcan-core](crates/libs/core/README.md)
- [vulcan-chain-parser](crates/libs/chain-parser/README.md)
- [vulcan-api](crates/services/api/README.md)
- [vulcan-worker](crates/services/worker/README.md)
- [vulcan-worker-orchestrator](crates/services/worker-orchestrator/README.md)
- [vulcan-workflow-trigger-processor](crates/services/workflow-trigger-processor/README.md)
- [vulcan-chain-parser-api](crates/services/chain-parser-api/README.md)
- [vulcan-chain-parser-cli](crates/services/chain-parser-cli/README.md)
