# Last Completed & Next Task

## Last Completed

**Worker-Controller Service** (`crates/services/worker-controller/`)

Implemented a Kubernetes-based auto-scaler that manages worker Deployment replicas based on job queue depth from the orchestrator.

Key deliverables:
- Queue metrics endpoint on orchestrator (`GET /queue/metrics`)
- Worker busy check endpoint (`GET /workers/{id}/busy`)
- Controller service with proportional scaling algorithm
- Scale-down cooldown to prevent flapping
- Environment-based configuration (no remote config)
- Unit tests for scaling algorithm and state management

## Next Task

**Add kind as Local Testing Environment**

Set up [kind](https://kind.sigs.k8s.io/) (Kubernetes in Docker) for local development and testing of the worker-controller.

Suggested scope:
- `kind` cluster configuration file
- Deployment manifests for worker-controller and worker
- Local orchestrator and PostgreSQL setup
- Integration test script to verify scaling behavior
- Documentation in worker-controller README

**Consideration:** kind could fully replace docker-compose as the single local testing environment. All services (PostgreSQL, orchestrator, workers) can run as pods, and this would:
- Eliminate maintaining two separate environments
- Ensure local testing matches production Kubernetes behavior
- Allow testing of K8s-specific features (RBAC, Deployments, scaling)

Tradeoff is slightly slower startup (~30-60s vs ~5s) and higher resource usage, but simplifies the overall setup.
