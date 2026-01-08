-- Remove indexes
DROP INDEX IF EXISTS idx_chains_commit;
DROP INDEX IF EXISTS idx_chains_repository;

-- Remove columns
ALTER TABLE chains
    DROP COLUMN IF EXISTS default_machine,
    DROP COLUMN IF EXISTS trigger_ref,
    DROP COLUMN IF EXISTS trigger,
    DROP COLUMN IF EXISTS branch,
    DROP COLUMN IF EXISTS commit_sha,
    DROP COLUMN IF EXISTS repository_url,
    DROP COLUMN IF EXISTS source_file_path;

-- Remove enum type
DROP TYPE IF EXISTS trigger_type;
