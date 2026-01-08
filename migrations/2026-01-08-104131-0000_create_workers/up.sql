CREATE TYPE worker_status AS ENUM ('active', 'suspended', 'error');

CREATE TABLE workers (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    status worker_status NOT NULL DEFAULT 'active',
    current_chain_id UUID,
    previous_chain_id UUID,
    next_chain_id UUID,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workers_tenant_id ON workers(tenant_id);
CREATE INDEX idx_workers_status ON workers(status);
