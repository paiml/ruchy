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
#[derive(Debug, Clone, PartialEq)]
pub struct SimConfig {
    /// Random seed for reproducibility (None = random)
    pub seed: Option<u64>,
    /// Maximum simulation steps
    pub max_steps: usize,
    /// Time step delta
    pub dt: f64,
    /// Whether to collect full event trace
    pub trace: bool,
    /// Stop condition: halt when invariant violated
    pub stop_on_violation: bool,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            seed: None,
            max_steps: 10_000,
            dt: 0.001,
            trace: false,
            stop_on_violation: true,
        }
    }
}

impl SimConfig {
    /// Create a config with a fixed seed for deterministic runs.
    pub fn deterministic(seed: u64) -> Self {
        Self {
            seed: Some(seed),
            ..Default::default()
        }
    }

    /// Set max steps.
    pub fn with_max_steps(mut self, steps: usize) -> Self {
        self.max_steps = steps;
        self
    }

    /// Set time step.
    pub fn with_dt(mut self, dt: f64) -> Self {
        self.dt = dt;
        self
    }

    /// Enable full event tracing.
    pub fn with_trace(mut self) -> Self {
        self.trace = true;
        self
    }
}

/// A single simulation event in the trace log.
#[derive(Debug, Clone, PartialEq)]
pub struct SimEvent {
    /// Simulation time when event occurred
    pub time: f64,
    /// Step number
    pub step: usize,
    /// Event description
    pub description: String,
}

/// Simulation result summary from `sim run`.
#[derive(Debug, Clone)]
pub struct SimResult {
    /// Number of steps executed
    pub steps: usize,
    /// Final simulation time
    pub final_time: f64,
    /// Whether all invariants held throughout
    pub invariants_ok: bool,
    /// Event trace (populated only if trace=true)
    pub events: Vec<SimEvent>,
    /// Exit reason
    pub exit_reason: SimExitReason,
}

/// Why the simulation stopped.
#[derive(Debug, Clone, PartialEq)]
pub enum SimExitReason {
    /// Completed all requested steps
    Completed,
    /// Stopped due to invariant violation
    InvariantViolation(String),
    /// Reached a steady state
    SteadyState,
    /// User-defined stop condition met
    StopCondition(String),
}

impl SimResult {
    /// Create a successful result with no events.
    pub fn completed(steps: usize, final_time: f64) -> Self {
        Self {
            steps,
            final_time,
            invariants_ok: true,
            events: Vec::new(),
            exit_reason: SimExitReason::Completed,
        }
    }

    /// Format as a human-readable summary line.
    pub fn summary(&self) -> String {
        let status = if self.invariants_ok { "OK" } else { "FAIL" };
        format!(
            "Simulation: {} steps, t={:.4}, invariants={}, exit={:?}",
            self.steps, self.final_time, status, self.exit_reason
        )
    }
}

/// Inspection snapshot of simulation state at a point in time.
#[derive(Debug, Clone)]
pub struct SimSnapshot {
    /// Current step
    pub step: usize,
    /// Current time
    pub time: f64,
    /// State variables and their values
    pub variables: Vec<(String, f64)>,
}

impl SimSnapshot {
    /// Get a variable value by name.
    pub fn get(&self, name: &str) -> Option<f64> {
        self.variables
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| *v)
    }
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
        assert!(!config.trace);
        assert!(config.stop_on_violation);
    }

    #[test]
    fn test_sim_config_deterministic() {
        let config = SimConfig::deterministic(42);
        assert_eq!(config.seed, Some(42));
        assert_eq!(config.max_steps, 10_000);
    }

    #[test]
    fn test_sim_config_builder() {
        let config = SimConfig::deterministic(123)
            .with_max_steps(500)
            .with_dt(0.01)
            .with_trace();
        assert_eq!(config.seed, Some(123));
        assert_eq!(config.max_steps, 500);
        assert!((config.dt - 0.01).abs() < f64::EPSILON);
        assert!(config.trace);
    }

    #[test]
    fn test_sim_result_completed() {
        let result = SimResult::completed(1000, 1.0);
        assert_eq!(result.steps, 1000);
        assert!(result.invariants_ok);
        assert_eq!(result.exit_reason, SimExitReason::Completed);
        assert!(result.events.is_empty());
    }

    #[test]
    fn test_sim_result_summary() {
        let result = SimResult::completed(500, 0.5);
        let summary = result.summary();
        assert!(summary.contains("500 steps"));
        assert!(summary.contains("OK"));
    }

    #[test]
    fn test_sim_snapshot_get() {
        let snap = SimSnapshot {
            step: 100,
            time: 0.1,
            variables: vec![
                ("x".to_string(), 1.5),
                ("y".to_string(), -0.3),
            ],
        };
        assert_eq!(snap.get("x"), Some(1.5));
        assert_eq!(snap.get("y"), Some(-0.3));
        assert_eq!(snap.get("z"), None);
    }

    #[test]
    fn test_sim_exit_reasons() {
        assert_eq!(SimExitReason::Completed, SimExitReason::Completed);
        let violation = SimExitReason::InvariantViolation("x > 0".to_string());
        assert_ne!(violation, SimExitReason::Completed);
    }
}
