-- Add execution status values to chain_status enum
ALTER TYPE chain_status ADD VALUE 'pending';
ALTER TYPE chain_status ADD VALUE 'running';
ALTER TYPE chain_status ADD VALUE 'completed';
ALTER TYPE chain_status ADD VALUE 'failed';

-- Add chain completion tracking fields
ALTER TABLE chains
    ADD COLUMN started_at TIMESTAMP,
    ADD COLUMN completed_at TIMESTAMP;
