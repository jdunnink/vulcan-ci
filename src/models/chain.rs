use uuid::Uuid;

#[derive(Debug)]
pub enum ChainStatus {
    Active,
    Suspended,
    Error,
}

pub struct Chain {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub status: ChainStatus, // computed from fragment status
    pub attempt: u32,
    pub fragments: Vec<Uuid>,
}
