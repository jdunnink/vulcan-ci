use diesel::pg::PgConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::models::worker::{NewWorker, Worker, WorkerStatus};
use crate::schema::workers;

use super::error::Result;

/// Repository trait for Worker entities.
pub trait WorkerRepository {
    /// Find a worker by its ID.
    fn find_by_id(&mut self, id: Uuid) -> Result<Option<Worker>>;

    /// Find all workers.
    fn find_all(&mut self) -> Result<Vec<Worker>>;

    /// Find all workers for a specific tenant.
    fn find_by_tenant(&mut self, tenant_id: Uuid) -> Result<Vec<Worker>>;

    /// Find workers by status.
    fn find_by_status(&mut self, status: WorkerStatus) -> Result<Vec<Worker>>;

    /// Create a new worker.
    fn create(&mut self, new_worker: NewWorker) -> Result<Worker>;

    /// Update an existing worker.
    fn update(&mut self, worker: &Worker) -> Result<Worker>;

    /// Delete a worker by ID.
    fn delete(&mut self, id: Uuid) -> Result<bool>;

    /// Count all workers.
    fn count(&mut self) -> Result<i64>;
}

/// `PostgreSQL` implementation of `WorkerRepository`.
pub struct PgWorkerRepository<'a> {
    conn: &'a mut PgConnection,
}

impl<'a> PgWorkerRepository<'a> {
    /// Creates a new `PgWorkerRepository` with the given connection.
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

impl WorkerRepository for PgWorkerRepository<'_> {
    fn find_by_id(&mut self, id: Uuid) -> Result<Option<Worker>> {
        let worker = workers::table
            .find(id)
            .first::<Worker>(self.conn)
            .optional()?;
        Ok(worker)
    }

    fn find_all(&mut self) -> Result<Vec<Worker>> {
        let results = workers::table.load::<Worker>(self.conn)?;
        Ok(results)
    }

    fn find_by_tenant(&mut self, tenant_id: Uuid) -> Result<Vec<Worker>> {
        let results = workers::table
            .filter(workers::tenant_id.eq(tenant_id))
            .load::<Worker>(self.conn)?;
        Ok(results)
    }

    fn find_by_status(&mut self, status: WorkerStatus) -> Result<Vec<Worker>> {
        let results = workers::table
            .filter(workers::status.eq(status))
            .load::<Worker>(self.conn)?;
        Ok(results)
    }

    fn create(&mut self, new_worker: NewWorker) -> Result<Worker> {
        let worker = diesel::insert_into(workers::table)
            .values(&new_worker)
            .returning(Worker::as_returning())
            .get_result(self.conn)?;
        Ok(worker)
    }

    fn update(&mut self, worker: &Worker) -> Result<Worker> {
        let updated = diesel::update(workers::table.find(worker.id))
            .set((
                workers::tenant_id.eq(&worker.tenant_id),
                workers::status.eq(&worker.status),
                workers::current_chain_id.eq(&worker.current_chain_id),
                workers::previous_chain_id.eq(&worker.previous_chain_id),
                workers::next_chain_id.eq(&worker.next_chain_id),
            ))
            .returning(Worker::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }

    fn delete(&mut self, id: Uuid) -> Result<bool> {
        let deleted = diesel::delete(workers::table.find(id)).execute(self.conn)?;
        Ok(deleted > 0)
    }

    fn count(&mut self) -> Result<i64> {
        let count = workers::table.count().get_result(self.conn)?;
        Ok(count)
    }
}
