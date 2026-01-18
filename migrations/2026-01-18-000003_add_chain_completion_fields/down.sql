-- Revert chain completion fields
ALTER TABLE chains
    DROP COLUMN IF EXISTS completed_at,
    DROP COLUMN IF EXISTS started_at;

-- Note: PostgreSQL does not support removing enum values directly.
-- The enum values 'pending', 'running', 'completed', 'failed' will remain.
-- To fully remove them, you would need to recreate the enum type.
