# Vulcan Worker Controller

Kubernetes-based auto-scaler that manages worker Deployment replicas based on job queue depth from the orchestrator.

## Status

**Implemented** - Core functionality complete.

## Overview

The worker-controller runs on client Kubernetes infrastructure and automatically scales worker Deployments up or down based on pending work in the orchestrator queue. It follows the pull-based model: the controller polls the orchestrator for metrics and makes scaling decisions locally.

```
Client K8s Cluster                    Vulcan Infrastructure
┌─────────────────────┐              ┌─────────────────────┐
│  worker-controller  │──metrics────►│    Orchestrator     │
│  (1 per machine_grp)│              │                     │
└─────────┬───────────┘              │  GET /queue/metrics │
          │ scale                    │                     │
          ▼                          └─────────────────────┘
┌─────────────────────┐
│  Worker Deployment  │──poll work──►
│  (N replicas)       │
└─────────────────────┘
```

## Running

### Prerequisites

- Kubernetes cluster with a worker Deployment already created
- `KUBECONFIG` set or running in-cluster with appropriate RBAC permissions
- Orchestrator service accessible from the cluster

### Local Development with kind

The easiest way to run the controller locally is with the kind cluster, which provides a real Kubernetes environment with all services:

```bash
# Start the full cluster (postgres, orchestrator, controller, workers)
task kind-up

# Verify everything is running
task kind-status

# View controller logs
task kind-logs -- worker-controller

# Run the scaling integration test
task test-scaling

# Rebuild and reload just the controller after code changes
task kind-load-controller

# Tear down
task kind-down
```

The kind cluster uses reduced polling (10s) and scale-down cooldown (60s) for faster dev feedback. See `k8s/04-controller.yaml` for the full configuration.

### With Cargo

```bash
ORCHESTRATOR_URL=http://orchestrator:3002 \
TENANT_ID=<uuid> \
MACHINE_GROUP=default \
DEPLOYMENT_NAME=vulcan-worker \
DEPLOYMENT_NAMESPACE=vulcan \
cargo run -p vulcan-worker-controller
```

### Required RBAC Permissions

The controller needs permissions to read and patch Deployments:

```yaml
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: vulcan-worker-controller
  namespace: vulcan
rules:
- apiGroups: ["apps"]
  resources: ["deployments"]
  verbs: ["get", "patch"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: vulcan-worker-controller
  namespace: vulcan
subjects:
- kind: ServiceAccount
  name: vulcan-worker-controller
  namespace: vulcan
roleRef:
  kind: Role
  name: vulcan-worker-controller
  apiGroup: rbac.authorization.k8s.io
```

## Configuration

All configuration is via environment variables.

### Required

| Variable | Description |
|----------|-------------|
| `ORCHESTRATOR_URL` | URL of the orchestrator service |
| `TENANT_ID` | Tenant UUID for metrics filtering |
| `MACHINE_GROUP` | Machine group to manage (matches worker `WORKER_GROUP`) |
| `DEPLOYMENT_NAME` | Name of the Kubernetes Deployment to scale |
| `DEPLOYMENT_NAMESPACE` | Namespace of the Deployment |

### Scaling Parameters (with defaults)

| Variable | Description | Default |
|----------|-------------|---------|
| `MIN_REPLICAS` | Minimum number of replicas to maintain | 0 |
| `MAX_REPLICAS` | Maximum number of replicas allowed | 10 |
| `TARGET_PENDING_PER_WORKER` | Target pending fragments per worker | 1.0 |
| `SCALE_DOWN_DELAY_SECONDS` | Cooldown before scaling down | 300 |
| `POLL_INTERVAL_SECONDS` | Interval between scaling checks | 30 |

### Example ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: worker-controller-config
  namespace: vulcan
data:
  ORCHESTRATOR_URL: "https://orchestrator.vulcan.example"
  TENANT_ID: "550e8400-e29b-41d4-a716-446655440000"
  MACHINE_GROUP: "gpu"
  DEPLOYMENT_NAME: "vulcan-worker-gpu"
  DEPLOYMENT_NAMESPACE: "vulcan"
  MIN_REPLICAS: "1"
  MAX_REPLICAS: "50"
  TARGET_PENDING_PER_WORKER: "1.0"
  SCALE_DOWN_DELAY_SECONDS: "300"
  POLL_INTERVAL_SECONDS: "30"
```

## Architecture

### Components

- **Config** (`config.rs`): Environment-based configuration with sensible defaults
- **Error** (`error.rs`): Error types using thiserror
- **Client** (`client/`): HTTP client for orchestrator queue metrics API
- **Scaler** (`scaler/`): Scaling algorithm and cooldown state management
- **Kubernetes** (`kubernetes/`): Deployment scaling via kube-rs
- **Controller** (`controller.rs`): Main reconciliation loop

### Scaling Algorithm

The controller uses a simple proportional scaling algorithm:

```
desired = ceil(pending_fragments / target_pending_per_worker)
desired = clamp(desired, min_replicas, max_replicas)
```

For example, with `target_pending_per_worker = 1.0`:
- 0 pending → 0 replicas (or min_replicas)
- 5 pending → 5 replicas
- 100 pending → 10 replicas (capped at max_replicas)

### Reconciliation Loop

Every `poll_interval_seconds`, the controller:

1. Fetches queue metrics from orchestrator (`GET /queue/metrics?machine_group=X`)
2. Gets current Deployment replica count via Kubernetes API
3. Calculates desired replicas using the scaling algorithm
4. If scaling up: immediately patches the Deployment
5. If scaling down: only if `scale_down_delay_seconds` has elapsed since last scale-down

### Scale-Down Cooldown

To prevent rapid scaling oscillation (flapping), the controller enforces a cooldown period after each scale-down operation. Scale-up operations are always immediate.

## Orchestrator API

The controller uses these orchestrator endpoints:

### GET /queue/metrics

Returns queue depth metrics for scaling decisions.

**Query Parameters:**
- `machine_group` (optional): Filter by machine group

**Response:**
```json
{
  "pending_fragments": 15,
  "running_fragments": 5,
  "active_workers": 5
}
```

### GET /workers/{id}/busy

Checks if a worker is currently executing a fragment. Useful for graceful shutdown via Kubernetes preStop hooks.

**Response:**
```json
{
  "busy": true,
  "fragment_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Graceful Shutdown

The controller handles SIGINT (Ctrl+C) for graceful shutdown:
- Stops the reconciliation loop
- Does not scale down workers on exit

### Worker preStop Hook

To prevent killing workers mid-execution during scale-down, add a preStop hook to your worker Deployment:

```yaml
lifecycle:
  preStop:
    exec:
      command:
        - /bin/sh
        - -c
        - |
          while curl -sf "${ORCHESTRATOR_URL}/workers/${WORKER_ID}/busy" | grep -q '"busy":true'; do
            sleep 5
          done
```

## Implemented Functionality

- Queue depth polling from orchestrator
- Proportional scaling algorithm
- Kubernetes Deployment scaling via kube-rs
- Scale-down cooldown to prevent flapping
- Graceful shutdown handling
- Environment-based configuration

## Future Improvements

- [ ] Authentication (API key / mTLS)
- [ ] Multi-tenant controller isolation
- [ ] Observability (Prometheus metrics, OpenTelemetry tracing)
- [ ] preStop hook integration for graceful worker termination
- [ ] Scale-to-zero with cold-start optimization
- [ ] Alternative scaling algorithms (step-based, percentage-based)
