//! Scaling algorithm implementation.

/// Configuration for the scaling algorithm.
#[derive(Debug, Clone)]
pub struct ScalingConfig {
    /// Minimum number of replicas.
    pub min_replicas: i32,
    /// Maximum number of replicas.
    pub max_replicas: i32,
    /// Target pending fragments per worker.
    pub target_pending_per_worker: f64,
}

/// Calculate the desired number of replicas based on pending work.
///
/// The formula is:
/// ```text
/// desired = ceil(pending_fragments / target_pending_per_worker)
/// result = clamp(desired, min_replicas, max_replicas)
/// ```
///
/// # Arguments
///
/// * `config` - Scaling configuration
/// * `pending_fragments` - Number of pending fragments in the queue
///
/// # Returns
///
/// The desired number of replicas, clamped to the configured range.
pub fn calculate_desired_replicas(config: &ScalingConfig, pending_fragments: i64) -> i32 {
    if config.target_pending_per_worker <= 0.0 {
        // Avoid division by zero
        return config.min_replicas;
    }

    let raw = (pending_fragments as f64 / config.target_pending_per_worker).ceil() as i32;
    raw.clamp(config.min_replicas, config.max_replicas)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> ScalingConfig {
        ScalingConfig {
            min_replicas: 0,
            max_replicas: 10,
            target_pending_per_worker: 1.0,
        }
    }

    #[test]
    fn test_zero_pending_returns_min() {
        let config = default_config();
        assert_eq!(calculate_desired_replicas(&config, 0), 0);
    }

    #[test]
    fn test_exact_multiple_of_target() {
        let config = default_config();
        assert_eq!(calculate_desired_replicas(&config, 5), 5);
    }

    #[test]
    fn test_rounds_up() {
        let config = ScalingConfig {
            min_replicas: 0,
            max_replicas: 10,
            target_pending_per_worker: 2.0,
        };
        assert_eq!(calculate_desired_replicas(&config, 3), 2); // ceil(3/2) = 2
        assert_eq!(calculate_desired_replicas(&config, 5), 3); // ceil(5/2) = 3
    }

    #[test]
    fn test_clamps_to_max() {
        let config = default_config();
        assert_eq!(calculate_desired_replicas(&config, 100), 10);
    }

    #[test]
    fn test_clamps_to_min() {
        let config = ScalingConfig {
            min_replicas: 2,
            max_replicas: 10,
            target_pending_per_worker: 1.0,
        };
        assert_eq!(calculate_desired_replicas(&config, 0), 2);
        assert_eq!(calculate_desired_replicas(&config, 1), 2);
    }

    #[test]
    fn test_zero_target_returns_min() {
        let config = ScalingConfig {
            min_replicas: 1,
            max_replicas: 10,
            target_pending_per_worker: 0.0,
        };
        assert_eq!(calculate_desired_replicas(&config, 100), 1);
    }
}
