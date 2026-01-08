CREATE TYPE chain_status AS ENUM ('active', 'suspended', 'error');

CREATE TABLE chains (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    status chain_status NOT NULL DEFAULT 'active',
    attempt INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chains_tenant_id ON chains(tenant_id);
CREATE INDEX idx_chains_status ON chains(status);

-- Add foreign key constraints to workers table now that chains exists
ALTER TABLE workers
    ADD CONSTRAINT fk_workers_current_chain
    FOREIGN KEY (current_chain_id) REFERENCES chains(id) ON DELETE SET NULL;

ALTER TABLE workers
    ADD CONSTRAINT fk_workers_previous_chain
    FOREIGN KEY (previous_chain_id) REFERENCES chains(id) ON DELETE SET NULL;

ALTER TABLE workers
    ADD CONSTRAINT fk_workers_next_chain
    FOREIGN KEY (next_chain_id) REFERENCES chains(id) ON DELETE SET NULL;
