//! Fragment scheduler for assigning work to workers.
//!
//! The scheduler determines which fragment a worker can execute based on:
//! 1. Machine group matching (or no group = any machine)
//! 2. Fragment dependencies being satisfied:
//!    - Sequential siblings: all previous siblings must be completed
//!    - Parallel siblings: can run immediately once parent is active

use diesel::PgConnection;
use tracing::debug;

use vulcan_core::models::fragment::Fragment;
use vulcan_core::models::worker::Worker;
use vulcan_core::repositories::{FragmentRepository, PgFragmentRepository};

use crate::error::Result;

/// Scheduler for finding executable fragments.
pub struct Scheduler<'a> {
    conn: &'a mut PgConnection,
}

impl<'a> Scheduler<'a> {
    /// Create a new scheduler with a database connection.
    pub fn new(conn: &'a mut PgConnection) -> Self {
        Self { conn }
    }

    /// Find work for a specific worker.
    ///
    /// Returns the first executable fragment that matches the worker's machine group.
    pub fn find_work_for_worker(self, worker: &Worker) -> Result<Option<Fragment>> {
        let mut repo = PgFragmentRepository::new(self.conn);

        // Get pending fragments matching worker's machine group
        let pending_fragments = repo.find_pending_by_machine(worker.machine_group.as_deref())?;

        // We need to check dependencies for each fragment
        // Since we need the connection for each check, we'll collect and process
        for fragment in pending_fragments {
            // Get siblings for dependency check
            let siblings = repo.find_siblings(fragment.chain_id, fragment.parent_fragment_id)?;

            // Find parent to check if parallel
            let is_parallel = if let Some(parent_id) = fragment.parent_fragment_id {
                let parent = repo.find_by_id(parent_id)?;
                parent.map(|p| p.is_parallel).unwrap_or(false)
            } else {
                // Top-level fragments are sequential by default
                false
            };

            if can_execute_with_siblings(&fragment, &siblings, is_parallel) {
                debug!(
                    fragment_id = %fragment.id,
                    worker_id = %worker.id,
                    "Found executable fragment for worker"
                );
                return Ok(Some(fragment));
            }
        }

        Ok(None)
    }
}

/// Check if a fragment can be executed given its siblings.
fn can_execute_with_siblings(fragment: &Fragment, siblings: &[Fragment], is_parallel: bool) -> bool {
    if is_parallel {
        // Parallel: can execute immediately (no dependency on siblings)
        true
    } else {
        // Sequential: all previous siblings must be completed
        for sibling in siblings {
            if sibling.sequence < fragment.sequence && !sibling.status.is_terminal() {
                debug!(
                    fragment_id = %fragment.id,
                    sibling_id = %sibling.id,
                    sibling_sequence = sibling.sequence,
                    "Fragment blocked by uncompleted sibling"
                );
                return false;
            }
        }
        true
    }
}
