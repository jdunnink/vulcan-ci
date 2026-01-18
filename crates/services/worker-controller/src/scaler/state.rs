//! State tracking for scale-down cooldown.

use std::time::{Duration, Instant};

/// Tracks scaling state including cooldown timers.
#[derive(Debug)]
pub struct ScalerState {
    /// Last time a scale-down operation was performed.
    last_scale_down: Option<Instant>,
    /// Current replica count.
    current_replicas: i32,
}

impl ScalerState {
    /// Create a new scaler state.
    pub fn new() -> Self {
        Self {
            last_scale_down: None,
            current_replicas: 0,
        }
    }

    /// Update the current replica count.
    pub fn set_current_replicas(&mut self, replicas: i32) {
        self.current_replicas = replicas;
    }

    /// Get the current replica count.
    pub fn current_replicas(&self) -> i32 {
        self.current_replicas
    }

    /// Check if scale-down cooldown has elapsed.
    ///
    /// # Arguments
    ///
    /// * `cooldown_seconds` - Required cooldown period in seconds
    ///
    /// # Returns
    ///
    /// `true` if cooldown has elapsed (or no previous scale-down), `false` otherwise.
    pub fn can_scale_down(&self, cooldown_seconds: i64) -> bool {
        match self.last_scale_down {
            None => true,
            Some(last) => {
                let elapsed = last.elapsed();
                elapsed >= Duration::from_secs(cooldown_seconds as u64)
            }
        }
    }

    /// Record that a scale-down operation was performed.
    pub fn record_scale_down(&mut self) {
        self.last_scale_down = Some(Instant::now());
    }

    /// Determine if scaling is needed and what action to take.
    ///
    /// # Arguments
    ///
    /// * `desired_replicas` - Desired replica count
    /// * `scale_down_delay_seconds` - Cooldown for scale-down
    ///
    /// # Returns
    ///
    /// `Some(new_replicas)` if scaling is needed, `None` otherwise.
    pub fn should_scale(
        &self,
        desired_replicas: i32,
        scale_down_delay_seconds: i64,
    ) -> Option<i32> {
        if desired_replicas == self.current_replicas {
            return None;
        }

        // Scale up is always allowed
        if desired_replicas > self.current_replicas {
            return Some(desired_replicas);
        }

        // Scale down requires cooldown to have elapsed
        if self.can_scale_down(scale_down_delay_seconds) {
            return Some(desired_replicas);
        }

        None
    }
}

impl Default for ScalerState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = ScalerState::new();
        assert_eq!(state.current_replicas(), 0);
        assert!(state.can_scale_down(300));
    }

    #[test]
    fn test_scale_up_always_allowed() {
        let mut state = ScalerState::new();
        state.set_current_replicas(2);

        // Scale up from 2 to 5 should be allowed
        assert_eq!(state.should_scale(5, 300), Some(5));
    }

    #[test]
    fn test_no_change_returns_none() {
        let mut state = ScalerState::new();
        state.set_current_replicas(3);

        assert_eq!(state.should_scale(3, 300), None);
    }

    #[test]
    fn test_scale_down_allowed_initially() {
        let mut state = ScalerState::new();
        state.set_current_replicas(5);

        // First scale-down should be allowed
        assert_eq!(state.should_scale(2, 300), Some(2));
    }

    #[test]
    fn test_scale_down_blocked_during_cooldown() {
        let mut state = ScalerState::new();
        state.set_current_replicas(5);
        state.record_scale_down();

        // Immediate scale-down should be blocked
        assert_eq!(state.should_scale(2, 300), None);
    }
}
