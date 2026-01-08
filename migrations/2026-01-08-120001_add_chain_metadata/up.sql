-- Add trigger type enum
CREATE TYPE trigger_type AS ENUM ('tag', 'push', 'pull_request', 'schedule', 'manual');

-- Add metadata columns to chains table
ALTER TABLE chains
    ADD COLUMN source_file_path TEXT,
    ADD COLUMN repository_url TEXT,
    ADD COLUMN commit_sha TEXT,
    ADD COLUMN branch TEXT,
    ADD COLUMN trigger trigger_type,
    ADD COLUMN trigger_ref TEXT,
    ADD COLUMN default_machine TEXT;

-- Add index for repository lookups
CREATE INDEX idx_chains_repository ON chains(repository_url);
CREATE INDEX idx_chains_commit ON chains(commit_sha);
