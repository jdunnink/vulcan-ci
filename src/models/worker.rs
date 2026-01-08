use uuid::Uuid;

#[derive(Debug)]
pub enum WorkerStatus {
    Active,
    Suspended,
    Error,
}

pub struct Worker {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub status: WorkerStatus,
    pub current_chain_id: Uuid,
    pub previous_chain_id: Uuid,
    pub next_chain_id: Uuid,
}
