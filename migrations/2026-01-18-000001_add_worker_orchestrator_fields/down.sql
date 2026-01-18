-- Revert worker orchestrator fields from workers table
DROP INDEX IF EXISTS idx_workers_machine_group;
DROP INDEX IF EXISTS idx_workers_last_heartbeat;

ALTER TABLE workers
    DROP COLUMN IF EXISTS current_fragment_id,
    DROP COLUMN IF EXISTS machine_group,
    DROP COLUMN IF EXISTS last_heartbeat_at;
