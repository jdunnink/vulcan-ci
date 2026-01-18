-- Revert fragment execution fields
DROP INDEX IF EXISTS idx_fragments_status;
DROP INDEX IF EXISTS idx_fragments_assigned_worker;

ALTER TABLE fragments
    DROP COLUMN IF EXISTS error_message,
    DROP COLUMN IF EXISTS exit_code,
    DROP COLUMN IF EXISTS completed_at,
    DROP COLUMN IF EXISTS started_at,
    DROP COLUMN IF EXISTS assigned_worker_id;

-- Note: PostgreSQL does not support removing enum values directly.
-- The enum values 'pending', 'running', 'completed', 'failed' will remain.
-- To fully remove them, you would need to recreate the enum type.
