use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::chain::{Chain, ChainStatus, NewChain};
use crate::schema::chains;

use super::error::Result;

/// Repository trait for Chain entities.
pub trait ChainRepository {
    /// Find a chain by its ID.
    fn find_by_id(&mut self, id: Uuid) -> Result<Option<Chain>>;

    /// Find all chains.
    fn find_all(&mut self) -> Result<Vec<Chain>>;

    /// Find all chains for a specific tenant.
    fn find_by_tenant(&mut self, tenant_id: Uuid) -> Result<Vec<Chain>>;

    /// Find chains by status.
    fn find_by_status(&mut self, status: ChainStatus) -> Result<Vec<Chain>>;

    /// Create a new chain.
    fn create(&mut self, new_chain: NewChain) -> Result<Chain>;

    /// Update an existing chain.
    fn update(&mut self, chain: &Chain) -> Result<Chain>;

    /// Delete a chain by ID.
    fn delete(&mut self, id: Uuid) -> Result<bool>;

    /// Count all chains.
    fn count(&mut self) -> Result<i64>;

    /// Update only the status of a chain.
    fn update_status(&mut self, chain_id: Uuid, status: ChainStatus) -> Result<Chain>;

    /// Mark a chain as started (sets status to Running and started_at timestamp).
    fn mark_started(&mut self, chain_id: Uuid) -> Result<Chain>;

    /// Mark a chain as completed (sets status to Completed and completed_at timestamp).
    fn mark_completed(&mut self, chain_id: Uuid) -> Result<Chain>;

    /// Mark a chain as failed (sets status to Failed and completed_at timestamp).
    fn mark_failed(&mut self, chain_id: Uuid) -> Result<Chain>;
}

/// `PostgreSQL` implementation of `ChainRepository`.
pub struct PgChainRepository<'a> {
    conn: &'a mut PgConnection,
}

impl<'a> PgChainRepository<'a> {
    /// Creates a new `PgChainRepository` with the given connection.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(conn: &'a mut PgConnection) -> Self {
        Self { conn }
    }

    /// Returns a mutable reference to the underlying connection.
    #[allow(clippy::missing_const_for_fn)]
    pub fn conn(&mut self) -> &mut PgConnection {
        self.conn
    }
}

impl ChainRepository for PgChainRepository<'_> {
    fn find_by_id(&mut self, id: Uuid) -> Result<Option<Chain>> {
        let chain = chains::table
            .find(id)
            .first::<Chain>(self.conn)
            .optional()?;
        Ok(chain)
    }

    fn find_all(&mut self) -> Result<Vec<Chain>> {
        let results = chains::table.load::<Chain>(self.conn)?;
        Ok(results)
    }

    fn find_by_tenant(&mut self, tenant_id: Uuid) -> Result<Vec<Chain>> {
        let results = chains::table
            .filter(chains::tenant_id.eq(tenant_id))
            .load::<Chain>(self.conn)?;
        Ok(results)
    }

    fn find_by_status(&mut self, status: ChainStatus) -> Result<Vec<Chain>> {
        let results = chains::table
            .filter(chains::status.eq(status))
            .load::<Chain>(self.conn)?;
        Ok(results)
    }

    fn create(&mut self, new_chain: NewChain) -> Result<Chain> {
        let chain = diesel::insert_into(chains::table)
            .values(&new_chain)
            .returning(Chain::as_returning())
            .get_result(self.conn)?;
        Ok(chain)
    }

    fn update(&mut self, chain: &Chain) -> Result<Chain> {
        let updated = diesel::update(chains::table.find(chain.id))
            .set((
                chains::tenant_id.eq(&chain.tenant_id),
                chains::status.eq(&chain.status),
                chains::attempt.eq(&chain.attempt),
            ))
            .returning(Chain::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }

    fn delete(&mut self, id: Uuid) -> Result<bool> {
        let deleted = diesel::delete(chains::table.find(id)).execute(self.conn)?;
        Ok(deleted > 0)
    }

    fn count(&mut self) -> Result<i64> {
        let count = chains::table.count().get_result(self.conn)?;
        Ok(count)
    }

    fn update_status(&mut self, chain_id: Uuid, status: ChainStatus) -> Result<Chain> {
        let updated = diesel::update(chains::table.find(chain_id))
            .set(chains::status.eq(status))
            .returning(Chain::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }

    fn mark_started(&mut self, chain_id: Uuid) -> Result<Chain> {
        let now = Utc::now().naive_utc();
        let updated = diesel::update(chains::table.find(chain_id))
            .set((
                chains::status.eq(ChainStatus::Running),
                chains::started_at.eq(Some(now)),
            ))
            .returning(Chain::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }

    fn mark_completed(&mut self, chain_id: Uuid) -> Result<Chain> {
        let now = Utc::now().naive_utc();
        let updated = diesel::update(chains::table.find(chain_id))
            .set((
                chains::status.eq(ChainStatus::Completed),
                chains::completed_at.eq(Some(now)),
            ))
            .returning(Chain::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }

    fn mark_failed(&mut self, chain_id: Uuid) -> Result<Chain> {
        let now = Utc::now().naive_utc();
        let updated = diesel::update(chains::table.find(chain_id))
            .set((
                chains::status.eq(ChainStatus::Failed),
                chains::completed_at.eq(Some(now)),
            ))
            .returning(Chain::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }
}
