//! Fragment scheduler for assigning work to workers.
//!
//! The scheduler determines which fragment a worker can execute based on:
//! 1. Machine group matching (or no group = any machine)
//! 2. Fragment dependencies being satisfied:
//!    - Sequential siblings: all previous siblings must be completed
//!    - Parallel siblings: can run immediately once parent is active
//!
//! Uses optimistic locking to prevent race conditions when multiple workers
//! request work simultaneously. This allows the system to scale to thousands
//! of workers without lock contention.

use diesel::PgConnection;
use tracing::{debug, trace};

use vulcan_core::models::fragment::Fragment;
use vulcan_core::models::worker::Worker;
use vulcan_core::repositories::{FragmentRepository, PgFragmentRepository};

use crate::error::Result;

/// Scheduler for finding and claiming executable fragments.
pub struct Scheduler<'a> {
    conn: &'a mut PgConnection,
}

impl<'a> Scheduler<'a> {
    /// Create a new scheduler with a database connection.
    pub fn new(conn: &'a mut PgConnection) -> Self {
        Self { conn }
    }

    /// Find and atomically claim work for a specific worker.
    ///
    /// Uses optimistic locking to prevent race conditions:
    /// 1. Find candidate pending fragments matching worker's machine group
    /// 2. Check dependencies for each candidate
    /// 3. Atomically try to claim the first eligible fragment
    /// 4. If claim fails (another worker got it), try the next candidate
    ///
    /// Returns the claimed fragment, or None if no work is available.
    pub fn find_and_claim_work(self, worker: &Worker) -> Result<Option<Fragment>> {
        let mut repo = PgFragmentRepository::new(self.conn);

        // Get pending fragments matching worker's machine group
        let pending_fragments = repo.find_pending_by_machine(worker.machine_group.as_deref())?;

        trace!(
            worker_id = %worker.id,
            pending_count = pending_fragments.len(),
            "Found pending fragments"
        );

        // Try to claim each eligible fragment
        for fragment in pending_fragments {
            // Check dependencies first (cheap operation)
            let siblings = repo.find_siblings(fragment.chain_id, fragment.parent_fragment_id)?;

            let is_parallel = if let Some(parent_id) = fragment.parent_fragment_id {
                let parent = repo.find_by_id(parent_id)?;
                parent.map(|p| p.is_parallel).unwrap_or(false)
            } else {
                false
            };

            if !can_execute_with_siblings(&fragment, &siblings, is_parallel) {
                trace!(
                    fragment_id = %fragment.id,
                    "Fragment not eligible due to dependencies"
                );
                continue;
            }

            // Try to atomically claim this fragment
            // This uses optimistic locking: only succeeds if still pending
            match repo.try_claim(fragment.id, worker.id)? {
                Some(claimed) => {
                    debug!(
                        fragment_id = %claimed.id,
                        worker_id = %worker.id,
                        "Successfully claimed fragment for worker"
                    );
                    return Ok(Some(claimed));
                }
                None => {
                    // Fragment was claimed by another worker, try next one
                    trace!(
                        fragment_id = %fragment.id,
                        worker_id = %worker.id,
                        "Fragment already claimed by another worker"
                    );
                    continue;
                }
            }
        }

        debug!(
            worker_id = %worker.id,
            "No claimable work available"
        );
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
                trace!(
                    fragment_id = %fragment.id,
                    sibling_id = %sibling.id,
                    sibling_sequence = sibling.sequence,
                    sibling_status = ?sibling.status,
                    "Fragment blocked by uncompleted sibling"
                );
                return false;
            }
        }
        true
    }
}
