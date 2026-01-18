-- Add worker orchestrator fields to workers table
ALTER TABLE workers
    ADD COLUMN last_heartbeat_at TIMESTAMP,
    ADD COLUMN machine_group TEXT,
    ADD COLUMN current_fragment_id UUID REFERENCES fragments(id) ON DELETE SET NULL;

CREATE INDEX idx_workers_last_heartbeat ON workers(last_heartbeat_at);
CREATE INDEX idx_workers_machine_group ON workers(machine_group);
