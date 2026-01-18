-- Add execution status values to fragment_status enum
ALTER TYPE fragment_status ADD VALUE 'pending';
ALTER TYPE fragment_status ADD VALUE 'running';
ALTER TYPE fragment_status ADD VALUE 'completed';
ALTER TYPE fragment_status ADD VALUE 'failed';

-- Add fragment execution tracking fields
ALTER TABLE fragments
    ADD COLUMN assigned_worker_id UUID REFERENCES workers(id) ON DELETE SET NULL,
    ADD COLUMN started_at TIMESTAMP,
    ADD COLUMN completed_at TIMESTAMP,
    ADD COLUMN exit_code INTEGER,
    ADD COLUMN error_message TEXT;

CREATE INDEX idx_fragments_assigned_worker ON fragments(assigned_worker_id);
-- Note: idx_fragments_status already exists from create_fragments migration
