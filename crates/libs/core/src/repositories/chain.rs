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
}
