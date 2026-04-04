//! Simular Bridge Module (Pillar 7: Simulation Engine)
//!
//! Thin wrappers around Simular for Ruchy stdlib.
//! Per ruchy-5.0-sovereign-platform.md Section 2: discrete-event simulation,
//! Monte Carlo, physics, and optimization domains.
//!
//! # Design
//! - Yoshida symplectic integrators for physics
//! - Jidoka guard injection for anomaly detection
//! - Deterministic PCG seeding for reproducibility
//!
//! # Feature Gate
//! Requires `--features simulation` to enable.

#[cfg(feature = "simulation")]
mod inner {
    pub use simular::*;
}

#[cfg(feature = "simulation")]
pub use inner::*;

/// Simulation configuration for the `sim run` command.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Maximum simulation steps
    pub max_steps: usize,
    /// Time step delta
    pub dt: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: None,
            max_steps: 10_000,
            dt: 0.001,
        }
    }
}

/// Simulation result summary.
#[derive(Debug)]
pub struct SimResult {
    /// Number of steps executed
    pub steps: usize,
    /// Final simulation time
    pub final_time: f64,
    /// Whether all invariants held
    pub invariants_ok: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sim_config_default() {
        let config = SimConfig::default();
        assert!(config.seed.is_none());
        assert_eq!(config.max_steps, 10_000);
        assert!((config.dt - 0.001).abs() < f64::EPSILON);
    }
}
