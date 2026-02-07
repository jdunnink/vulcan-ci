#!/usr/bin/env bash
# Integration test: verifies worker-controller scales workers up and down
# based on pending fragments in the orchestrator queue.
#
# Prerequisites: kind cluster running via `task kind-up`
set -euo pipefail

ORCHESTRATOR_URL="http://localhost:3002"
NAMESPACE="vulcan"
DEPLOYMENT="vulcan-worker"
TENANT_ID="00000000-0000-0000-0000-000000000001"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}[PASS]${NC} $1"; }
fail() { echo -e "${RED}[FAIL]${NC} $1"; exit 1; }
info() { echo -e "${YELLOW}[INFO]${NC} $1"; }

# Poll a command until it succeeds or times out
# Usage: poll <timeout_secs> <description> <command...>
poll() {
    local timeout=$1 desc=$2; shift 2
    local elapsed=0
    while ! "$@" 2>/dev/null; do
        sleep 3
        elapsed=$((elapsed + 3))
        if [ "$elapsed" -ge "$timeout" ]; then
            fail "$desc (timed out after ${timeout}s)"
        fi
    done
    pass "$desc"
}

get_replicas() {
    kubectl get deployment "$DEPLOYMENT" -n "$NAMESPACE" -o jsonpath='{.spec.replicas}'
}

get_pending() {
    curl -sf "${ORCHESTRATOR_URL}/queue/metrics?machine_group=default" | python3 -c "import sys,json; print(json.load(sys.stdin)['pending_fragments'])"
}

get_running() {
    curl -sf "${ORCHESTRATOR_URL}/queue/metrics?machine_group=default" | python3 -c "import sys,json; print(json.load(sys.stdin)['running_fragments'])"
}

echo "==========================================="
echo " Vulcan CI - Scaling Integration Test"
echo "==========================================="
echo ""

# Step 1: Health check
info "Checking orchestrator health..."
curl -sf "${ORCHESTRATOR_URL}/health" > /dev/null || fail "Orchestrator not reachable at ${ORCHESTRATOR_URL}"
pass "Orchestrator is healthy"

# Step 2: Verify initial state
info "Checking initial queue state..."
pending=$(get_pending)
[ "$pending" -eq 0 ] || fail "Expected 0 pending fragments, got $pending"
pass "Queue is empty (0 pending)"

info "Checking initial worker replicas..."
replicas=$(get_replicas)
[ "$replicas" -eq 0 ] || fail "Expected 0 worker replicas, got $replicas"
pass "Worker deployment at 0 replicas"

# Step 3: Insert test work
info "Inserting test chain with 3 fragments..."
kubectl exec deployment/postgres -n "$NAMESPACE" -- psql -U vulcan -d vulcan_ci -q -c "
    INSERT INTO chains (id, tenant_id, status)
    VALUES ('aaaaaaaa-0000-0000-0000-000000000001', '${TENANT_ID}', 'active');

    INSERT INTO fragments (id, chain_id, status, type, run_script, machine, sequence)
    VALUES
        ('bbbbbbbb-0000-0000-0000-000000000001', 'aaaaaaaa-0000-0000-0000-000000000001', 'pending', 'inline', 'echo hello && sleep 5', 'default', 0),
        ('bbbbbbbb-0000-0000-0000-000000000002', 'aaaaaaaa-0000-0000-0000-000000000001', 'pending', 'inline', 'echo hello && sleep 5', 'default', 1),
        ('bbbbbbbb-0000-0000-0000-000000000003', 'aaaaaaaa-0000-0000-0000-000000000001', 'pending', 'inline', 'echo hello && sleep 5', 'default', 2);
" || fail "Failed to insert test data"
pass "Inserted 1 chain with 3 pending fragments"

# Step 4: Verify queue sees the fragments
info "Waiting for queue metrics to reflect pending fragments..."
poll 30 "Queue shows pending fragments" bash -c '[ "$(curl -sf "'"${ORCHESTRATOR_URL}"'/queue/metrics?machine_group=default" | python3 -c "import sys,json; print(json.load(sys.stdin)['"'"'pending_fragments'"'"'])")" -ge 3 ]'

# Step 5: Wait for scale-up
info "Waiting for worker-controller to scale up workers..."
poll 90 "Workers scaled up" bash -c '[ "$(kubectl get deployment '"$DEPLOYMENT"' -n '"$NAMESPACE"' -o jsonpath='"'"'{.spec.replicas}'"'"')" -gt 0 ]'

scaled_to=$(get_replicas)
info "Workers scaled to $scaled_to replicas"

# Step 6: Wait for worker pods to be ready
info "Waiting for worker pods to become ready..."
kubectl wait --namespace "$NAMESPACE" --for=condition=ready pod -l app=vulcan-worker --timeout=120s || fail "Worker pods not ready"
pass "Worker pods are ready"

# Step 7: Wait for work to complete
info "Waiting for all fragments to complete..."
poll 180 "All work completed" bash -c '
    pending=$(curl -sf "'"${ORCHESTRATOR_URL}"'/queue/metrics?machine_group=default" | python3 -c "import sys,json; print(json.load(sys.stdin)['"'"'pending_fragments'"'"'])")
    running=$(curl -sf "'"${ORCHESTRATOR_URL}"'/queue/metrics?machine_group=default" | python3 -c "import sys,json; print(json.load(sys.stdin)['"'"'running_fragments'"'"'])")
    [ "$pending" -eq 0 ] && [ "$running" -eq 0 ]
'

# Step 8: Wait for scale-down
info "Waiting for worker-controller to scale down (cooldown ~60s)..."
poll 180 "Workers scaled back to 0" bash -c '[ "$(kubectl get deployment '"$DEPLOYMENT"' -n '"$NAMESPACE"' -o jsonpath='"'"'{.spec.replicas}'"'"')" -eq 0 ]'

# Cleanup test data
info "Cleaning up test data..."
kubectl exec deployment/postgres -n "$NAMESPACE" -- psql -U vulcan -d vulcan_ci -q -c "
    DELETE FROM chains WHERE id = 'aaaaaaaa-0000-0000-0000-000000000001';
" 2>/dev/null || true

echo ""
echo "==========================================="
echo -e " ${GREEN}All scaling tests passed!${NC}"
echo "==========================================="
