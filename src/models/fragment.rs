use uuid::Uuid;

#[derive(Debug)]
pub enum FragmentStatus {
    Active,
    Suspended,
    Error,
}

pub struct Fragment {
    pub id: Uuid,
    pub chain_id: Uuid,
    pub attempt: u32,
    pub status: FragmentStatus,
}
