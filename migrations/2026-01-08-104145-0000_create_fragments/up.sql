CREATE TYPE fragment_status AS ENUM ('active', 'suspended', 'error');

CREATE TABLE fragments (
    id UUID PRIMARY KEY,
    chain_id UUID NOT NULL REFERENCES chains(id) ON DELETE CASCADE,
    attempt INTEGER NOT NULL DEFAULT 1,
    status fragment_status NOT NULL DEFAULT 'active',
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_fragments_chain_id ON fragments(chain_id);
CREATE INDEX idx_fragments_status ON fragments(status);
