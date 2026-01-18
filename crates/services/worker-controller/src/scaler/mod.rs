//! Scaling logic module.

pub mod algorithm;
pub mod state;

pub use algorithm::{calculate_desired_replicas, ScalingConfig};
pub use state::ScalerState;
