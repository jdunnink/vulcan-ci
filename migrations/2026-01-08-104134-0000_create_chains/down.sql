ALTER TABLE workers DROP CONSTRAINT IF EXISTS fk_workers_current_chain;
ALTER TABLE workers DROP CONSTRAINT IF EXISTS fk_workers_previous_chain;
ALTER TABLE workers DROP CONSTRAINT IF EXISTS fk_workers_next_chain;

DROP TABLE IF EXISTS chains;
DROP TYPE IF EXISTS chain_status;
