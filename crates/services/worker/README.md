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
| `SANDBOX_ENABLED` | Enable bubblewrap sandboxing | No | true |
| `SANDBOX_MEMORY_LIMIT` | Memory limit for sandbox (e.g., "512M") | No | 512M |
| `SANDBOX_NETWORK` | Allow network access in sandbox | No | false |
| `SANDBOX_SCRATCH_DIR` | Scratch directory for script execution | No | /scratch |

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
- Bubblewrap sandbox for script isolation

## Security Model

The worker implements defense-in-depth with multiple security layers:

### Layer 1: Container Hardening (Docker)

- **Non-root user**: Worker runs as `vulcan` (UID 1000), not root
- **Read-only filesystem**: Root filesystem is read-only
- **Dropped capabilities**: All capabilities dropped except:
  - `KILL`: Required to terminate timed-out scripts
  - `SYS_ADMIN`: Required for bubblewrap to create namespaces
- **No privilege escalation**: `no-new-privileges` prevents setuid binaries
- **Resource limits**: CPU, memory, and PID limits prevent DoS
- **tmpfs mounts**: `/scratch` and `/tmp` are isolated tmpfs filesystems

### Layer 2: Bubblewrap Sandbox (Script Execution)

Scripts run inside a bubblewrap (bwrap) sandbox with:

- **PID namespace**: Scripts can't see host processes
- **Network namespace**: Network isolated by default (configurable)
- **UTS namespace**: Hostname isolated ("sandbox")
- **IPC namespace**: Inter-process communication isolated
- **Mount namespace**: Isolated filesystem view with:
  - Read-only binds: `/usr`, `/lib`, `/lib64`, `/bin`, `/sbin`
  - Minimal `/etc`: Only `passwd`, `group`, `hosts`, `resolv.conf`
  - Fresh `/dev` and `/proc`
  - tmpfs for `/tmp` and `/run`
  - Writable `/work` directory (bind-mounted from `/scratch`)
- **Clean environment**: Only `PATH`, `HOME`, `TMPDIR` set
- **Session isolation**: New session prevents terminal access
- **Die with parent**: Sandbox killed if worker dies

### Security Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│ Host System                                                      │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │ Docker Container (read-only, non-root, limited caps)        │ │
│  │  ┌───────────────────────────────────────────────────────┐ │ │
│  │  │ Bubblewrap Sandbox (namespaces, isolated filesystem)   │ │ │
│  │  │  ┌─────────────────────────────────────────────────┐  │ │ │
│  │  │  │ User Script (minimal env, /work only writable)  │  │ │ │
│  │  │  └─────────────────────────────────────────────────┘  │ │ │
│  │  └───────────────────────────────────────────────────────┘ │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### What's Prevented

- **Privilege escalation**: No setuid, dropped caps, non-root user
- **Host process access**: PID namespace isolation
- **Network exfiltration**: Network namespace (when disabled)
- **Filesystem tampering**: Read-only mounts, isolated scratch space
- **Resource exhaustion**: CPU, memory, PID limits
- **Escape via terminal**: New session, no TTY
- **Persistence**: tmpfs scratch cleared between executions

## Future Improvements

- stdout/stderr streaming to orchestrator
- Environment variable injection from chain config
- Secret injection (vault integration)
- OpenTelemetry integration for distributed tracing
- Custom seccomp profiles for additional syscall filtering
