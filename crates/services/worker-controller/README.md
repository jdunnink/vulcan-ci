
  Worker-Controller Architecture Plan

  Deployment Model

  ┌─────────────────────────────────────────┐
  │           Vulcan Infrastructure         │
  │  ┌───────────────────────────────────┐  │
  │  │         Orchestrator              │  │
  │  │  - Job queue management           │  │
  │  │  - Work distribution              │  │
  │  │  - Fragment scheduling            │  │
  │  └───────────────────────────────────┘  │
  └─────────────────────────────────────────┘
                      ▲
                      │ HTTPS (polling)
                      │
  ┌─────────────────────────────────────────┐
  │         Client Infrastructure           │
  │  ┌───────────────────────────────────┐  │
  │  │       Worker-Controller           │  │
  │  │  - Monitors queue depth           │  │
  │  │  - Scales workers up/down         │  │
  │  │  - Manages worker lifecycle       │  │
  │  └───────────────────────────────────┘  │
  │              │                          │
  │              │ spawns/terminates        │
  │              ▼                          │
  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐       │
  │  │ W1  │ │ W2  │ │ W3  │ │ ... │       │
  │  └─────┘ └─────┘ └─────┘ └─────┘       │
  │     │       │       │       │           │
  │     └───────┴───────┴───────┘           │
  │              │ poll for work            │
  │              ▼                          │
  └──────────────────────────────────────────
                      │
                      │ HTTPS (to orchestrator)
                      ▼

  Key Components

  1. New Orchestrator Endpoint
  Add GET /queue/metrics to expose queue depth per machine group:
  QueueMetricsResponse {
      machine_group: Option<String>,
      pending_fragments: u32,
      running_fragments: u32,
      active_workers: u32,
  }

  2. Worker-Controller Service (crates/services/worker-controller/)
  - ScaleSetConfig: min_replicas, max_replicas, machine_group, scale_up_threshold,
  scale_down_threshold, cooldown_period
  - MetricsPoller: Periodically queries orchestrator for queue depth
  - ScalingEngine: Decides when to scale based on algorithm
  - WorkerManager: Spawns/terminates worker processes (abstracted for different
  runtimes)

  3. Scaling Algorithms
  - Queue-depth based: Scale up when pending > workers * factor, scale down when
  idle
  - Percentage-based: Maintain X% headroom above current queue
  - Step scaling: Define thresholds (e.g., 0-5 pending = 1 worker, 6-20 = 3
  workers, etc.)

  4. Worker Manager Backends
  Abstract worker lifecycle management to support:
  - Process: Direct subprocess spawning (simple/local)
  - Docker: Container management via Docker API
  - Kubernetes: Pod management via K8s API (like ARC)

  Questions to Clarify

  Before I create a detailed implementation plan, a few questions:

  1. Initial backend priority - Which worker manager backend should we implement
  first? (Process/Docker/Kubernetes)
  2. Controller ↔ Orchestrator auth - Should the controller authenticate
  differently than workers? (API key, mTLS, etc.)
  3. Multi-group support - Should one controller manage multiple machine groups, or
   one controller per group?
  4. Scaling config source - Should the scaling config be:
    - Local config file on client infra
    - Fetched from orchestrator (centralized management)
    - Both (local override of central defaults)

  Would you like to discuss these points, or should I proceed with reasonable
  defaults and create a detailed implementation plan?

    ┌──────────┬─────────────────────────────────────────┐
  │ Decision │                 Choice                  │
  ├──────────┼─────────────────────────────────────────┤
  │ Backend  │ Kubernetes (pod management via kube-rs) │
  ├──────────┼─────────────────────────────────────────┤
  │ Topology │ One controller per machine group        │
  ├──────────┼─────────────────────────────────────────┤
  │ Config   │ Orchestrator defaults + local override  │
  ├──────────┼─────────────────────────────────────────┤
  │ Auth     │ Defer to roadmap                        │
  └──────────┴─────────────────────────────────────────┘
  Refined Architecture

  ┌─────────────────────────────────────────────────────────┐
  │                 Vulcan Infrastructure                   │
  │  ┌───────────────────────────────────────────────────┐  │
  │  │              Orchestrator                         │  │
  │  │  GET /queue/metrics?machine_group=X  (new)       │  │
  │  │  GET /scale-sets/{id}/config         (new)       │  │
  │  │  POST /work/request                  (existing)  │  │
  │  └───────────────────────────────────────────────────┘  │
  └─────────────────────────────────────────────────────────┘
                            ▲
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
  ┌───────┴───────┐ ┌───────┴───────┐ ┌───────┴───────┐
  │ Client K8s A  │ │ Client K8s B  │ │ Client K8s C  │
  │               │ │               │ │               │
  │ ┌───────────┐ │ │ ┌───────────┐ │ │ ┌───────────┐ │
  │ │Controller │ │ │ │Controller │ │ │ │Controller │ │
  │ │ (gpu)     │ │ │ │ (default) │ │ │ │ (arm64)   │ │
  │ └─────┬─────┘ │ │ └─────┬─────┘ │ │ └─────┬─────┘ │
  │       │       │ │       │       │ │       │       │
  │   ┌───┴───┐   │ │   ┌───┴───┐   │ │   ┌───┴───┐   │
  │   │Worker │   │ │   │Worker │   │ │   │Worker │   │
  │   │ Pods  │   │ │   │ Pods  │   │ │   │ Pods  │   │
  │   └───────┘   │ │   └───────┘   │ │   └───────┘   │
  └───────────────┘ └───────────────┘ └───────────────┘

  Implementation Components

  1. New Orchestrator Endpoints

  Queue Metrics - for controller scaling decisions:
  GET /queue/metrics?machine_group=gpu

  QueueMetricsResponse {
      machine_group: Option<String>,
      pending_fragments: u32,
      running_fragments: u32,
      assigned_workers: u32,
      oldest_pending_seconds: Option<u32>,
  }

  Scale Set Config - centralized defaults:
  GET /scale-sets/{tenant_id}/{machine_group}/config

  ScaleSetConfigResponse {
      min_replicas: u32,
      max_replicas: u32,
      target_pending_per_worker: f32,  // e.g., 1.0 = 1 pending job per worker
      scale_down_delay_seconds: u32,
      poll_interval_seconds: u32,
  }

  2. Worker-Controller Service

  crates/services/worker-controller/
  ├── src/
  │   ├── main.rs              # Entry point
  │   ├── lib.rs
  │   ├── config.rs            # Local config + merge logic
  │   ├── controller.rs        # Main reconciliation loop
  │   ├── scaling/
  │   │   ├── mod.rs
  │   │   ├── engine.rs        # Scaling decision logic
  │   │   └── algorithms.rs    # Different scaling strategies
  │   ├── kubernetes/
  │   │   ├── mod.rs
  │   │   ├── client.rs        # kube-rs wrapper
  │   │   └── worker_set.rs    # Worker pod management
  │   └── orchestrator/
  │       ├── mod.rs
  │       └── client.rs        # Metrics + config fetching
  └── Cargo.toml

  3. Reconciliation Loop (K8s controller pattern)

  every poll_interval:
      1. Fetch queue metrics from orchestrator
      2. Fetch remote config (cache with TTL)
      3. Merge with local overrides
      4. Get current worker pod count
      5. Calculate desired replicas:
         desired = ceil(pending_fragments / target_pending_per_worker)
         desired = clamp(desired, min_replicas, max_replicas)
      6. If desired != current AND cooldown elapsed:
         - Scale deployment to desired
         - Reset cooldown timer

  4. Local Config Override (ConfigMap or env)

  # worker-controller-config.yaml
  orchestrator_url: "https://vulcan.example.com"
  tenant_id: "uuid-here"
  machine_group: "gpu"

  # Local overrides (optional)
  overrides:
    min_replicas: 2        # override orchestrator default
    max_replicas: 50       # override orchestrator default
    # target_pending_per_worker: use orchestrator default

  ---
  A few follow-up points to discuss:

  1. Worker pod template - Should the controller manage a Deployment, or create
  individual Pods? Deployment is simpler but less flexible. Individual pods allow
  per-job labeling.
  2. Scale-to-zero - Should we support min_replicas: 0? This saves cost but adds
  cold-start latency. Could have a "scale from zero delay" before first pod is
  ready.
  3. Graceful scale-down - When scaling down, we should avoid killing workers
  mid-execution. Options:
    - Mark pods for deletion, let them finish current work and self-terminate
    - Use K8s preStop hooks
    - Controller tracks which workers are idle

  Thoughts on these?