-- Add fragment type enum (only inline and group - imports are resolved at parse time)
CREATE TYPE fragment_type AS ENUM ('inline', 'group');

-- Add new columns to fragments table
ALTER TABLE fragments
    ADD COLUMN parent_fragment_id UUID REFERENCES fragments(id) ON DELETE CASCADE,
    ADD COLUMN sequence INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN type fragment_type NOT NULL DEFAULT 'inline',
    ADD COLUMN run_script TEXT,
    ADD COLUMN machine TEXT,
    ADD COLUMN is_parallel BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN condition TEXT,
    ADD COLUMN source_url TEXT;  -- tracks where this fragment was imported from (NULL if inline)

-- Add indexes for hierarchy and ordering
CREATE INDEX idx_fragments_parent_id ON fragments(parent_fragment_id);
CREATE INDEX idx_fragments_sequence ON fragments(chain_id, parent_fragment_id, sequence);

-- Add constraint: inline fragments must have run_script, group fragments must not
ALTER TABLE fragments
    ADD CONSTRAINT chk_fragment_type_fields CHECK (
        (type = 'inline' AND run_script IS NOT NULL) OR
        (type = 'group' AND run_script IS NULL)
    );
