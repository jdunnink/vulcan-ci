use chrono::NaiveDateTime;
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

    /// Find workers whose heartbeat is older than the given threshold (dead workers).
    fn find_dead_workers(&mut self, threshold: NaiveDateTime) -> Result<Vec<Worker>>;

    /// Find idle workers (active status, no current fragment) optionally filtered by machine group.
    fn find_idle_by_machine_group(&mut self, machine_group: Option<&str>) -> Result<Vec<Worker>>;

    /// Update a worker's heartbeat timestamp to now.
    fn update_heartbeat(&mut self, worker_id: Uuid) -> Result<Worker>;

    /// Assign a fragment to a worker.
    fn assign_fragment(&mut self, worker_id: Uuid, fragment_id: Uuid) -> Result<Worker>;

    /// Clear a worker's current fragment assignment.
    fn clear_assignment(&mut self, worker_id: Uuid) -> Result<Worker>;
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
                workers::last_heartbeat_at.eq(&worker.last_heartbeat_at),
                workers::machine_group.eq(&worker.machine_group),
                workers::current_fragment_id.eq(&worker.current_fragment_id),
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

    fn find_dead_workers(&mut self, threshold: NaiveDateTime) -> Result<Vec<Worker>> {
        let results = workers::table
            .filter(workers::status.eq(WorkerStatus::Active))
            .filter(workers::last_heartbeat_at.lt(threshold))
            .load::<Worker>(self.conn)?;
        Ok(results)
    }

    fn find_idle_by_machine_group(&mut self, machine_group: Option<&str>) -> Result<Vec<Worker>> {
        let mut query = workers::table
            .filter(workers::status.eq(WorkerStatus::Active))
            .filter(workers::current_fragment_id.is_null())
            .into_boxed();

        if let Some(group) = machine_group {
            query = query.filter(workers::machine_group.eq(group));
        }

        let results = query.load::<Worker>(self.conn)?;
        Ok(results)
    }

    fn update_heartbeat(&mut self, worker_id: Uuid) -> Result<Worker> {
        let now = chrono::Utc::now().naive_utc();
        let updated = diesel::update(workers::table.find(worker_id))
            .set(workers::last_heartbeat_at.eq(now))
            .returning(Worker::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }

    fn assign_fragment(&mut self, worker_id: Uuid, fragment_id: Uuid) -> Result<Worker> {
        let updated = diesel::update(workers::table.find(worker_id))
            .set(workers::current_fragment_id.eq(Some(fragment_id)))
            .returning(Worker::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }

    fn clear_assignment(&mut self, worker_id: Uuid) -> Result<Worker> {
        let updated = diesel::update(workers::table.find(worker_id))
            .set(workers::current_fragment_id.eq(None::<Uuid>))
            .returning(Worker::as_returning())
            .get_result(self.conn)?;
        Ok(updated)
    }
}
