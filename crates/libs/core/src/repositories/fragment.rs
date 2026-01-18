use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::fragment::{Fragment, FragmentStatus, NewFragment};
use crate::schema::fragments;

use super::error::Result;

/// Repository trait for Fragment entities.
pub trait FragmentRepository {
    /// Find a fragment by its ID.
    fn find_by_id(&mut self, id: Uuid) -> Result<Option<Fragment>>;

    /// Find all fragments.
    fn find_all(&mut self) -> Result<Vec<Fragment>>;

    /// Find all fragments for a specific chain.
    fn find_by_chain(&mut self, chain_id: Uuid) -> Result<Vec<Fragment>>;

    /// Find fragments by status.
    fn find_by_status(&mut self, status: FragmentStatus) -> Result<Vec<Fragment>>;

    /// Create a new fragment.
    fn create(&mut self, new_fragment: NewFragment) -> Result<Fragment>;

    /// Create multiple fragments at once.
    fn create_many(&mut self, new_fragments: Vec<NewFragment>) -> Result<Vec<Fragment>>;

    /// Update an existing fragment.
    fn update(&mut self, fragment: &Fragment) -> Result<Fragment>;

    /// Delete a fragment by ID.
    fn delete(&mut self, id: Uuid) -> Result<bool>;

    /// Delete all fragments for a chain.
    fn delete_by_chain(&mut self, chain_id: Uuid) -> Result<usize>;

    /// Count all fragments.
    fn count(&mut self) -> Result<i64>;

    /// Count fragments for a specific chain.
    fn count_by_chain(&mut self, chain_id: Uuid) -> Result<i64>;
}

/// `PostgreSQL` implementation of `FragmentRepository`.
pub struct PgFragmentRepository<'a> {
    conn: &'a mut PgConnection,
}

impl<'a> PgFragmentRepository<'a> {
    /// Creates a new `PgFragmentRepository` with the given connection.
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

impl FragmentRepository for PgFragmentRepository<'_> {
    fn find_by_id(&mut self, id: Uuid) -> Result<Option<Fragment>> {
        let fragment = fragments::table
            .find(id)
            .first::<Fragment>(self.conn)
            .optional()?;
        Ok(fragment)
    }

    fn find_all(&mut self) -> Result<Vec<Fragment>> {
        let results = fragments::table.load::<Fragment>(self.conn)?;
        Ok(results)
    }

    fn find_by_chain(&mut self, chain_id: Uuid) -> Result<Vec<Fragment>> {
        let results = fragments::table
            .filter(fragments::chain_id.eq(chain_id))
            .load::<Fragment>(self.conn)?;
        Ok(results)
    }

    fn find_by_status(&mut self, status: FragmentStatus) -> Result<Vec<Fragment>> {
        let results = fragments::table
            .filter(fragments::status.eq(status))
            .load::<Fragment>(self.conn)?;
        Ok(results)
    }

    fn create(&mut self, new_fragment: NewFragment) -> Result<Fragment> {
        let fragment = diesel::insert_into(fragments::table)
            .values(&new_fragment)
            .returning(Fragment::as_returning())
            .get_result(self.conn)?;
        Ok(fragment)
    }

    fn create_many(&mut self, new_fragments: Vec<NewFragment>) -> Result<Vec<Fragment>> {
        let created = diesel::insert_into(fragments::table)
            .values(&new_fragments)
            .returning(Fragment::as_returning())
            .get_results(self.conn)?;
        Ok(created)
    }

    fn update(&mut self, fragment: &Fragment) -> Result<Fragment> {
        let updated = diesel::update(fragments::table.find(fragment.id))
            .set((
                fragments::chain_id.eq(&fragment.chain_id),
                fragments::attempt.eq(&fragment.attempt),
                fragments::status.eq(&fragment.status),
            ))
            .returning(Fragment::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }

    fn delete(&mut self, id: Uuid) -> Result<bool> {
        let deleted = diesel::delete(fragments::table.find(id)).execute(self.conn)?;
        Ok(deleted > 0)
    }

    fn delete_by_chain(&mut self, chain_id: Uuid) -> Result<usize> {
        let deleted = diesel::delete(fragments::table.filter(fragments::chain_id.eq(chain_id)))
            .execute(self.conn)?;
        Ok(deleted)
    }

    fn count(&mut self) -> Result<i64> {
        let count = fragments::table.count().get_result(self.conn)?;
        Ok(count)
    }

    fn count_by_chain(&mut self, chain_id: Uuid) -> Result<i64> {
        let count = fragments::table
            .filter(fragments::chain_id.eq(chain_id))
            .count()
            .get_result(self.conn)?;
        Ok(count)
    }
}
