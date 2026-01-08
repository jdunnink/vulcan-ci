-- Remove constraint
ALTER TABLE fragments DROP CONSTRAINT IF EXISTS chk_fragment_type_fields;

-- Remove indexes
DROP INDEX IF EXISTS idx_fragments_sequence;
DROP INDEX IF EXISTS idx_fragments_parent_id;

-- Remove columns
ALTER TABLE fragments
    DROP COLUMN IF EXISTS source_url,
    DROP COLUMN IF EXISTS condition,
    DROP COLUMN IF EXISTS is_parallel,
    DROP COLUMN IF EXISTS machine,
    DROP COLUMN IF EXISTS run_script,
    DROP COLUMN IF EXISTS type,
    DROP COLUMN IF EXISTS sequence,
    DROP COLUMN IF EXISTS parent_fragment_id;

-- Remove enum type
DROP TYPE IF EXISTS fragment_type;
