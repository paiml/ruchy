//! Unified Training Loop for Oracle
//!
//! Implements the continuous learning loop with visual feedback.
//! Integrates curriculum learning, drift detection, and automatic retraining.
//!
//! # References
//! - Spec: docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md §13

use super::{
    andon::{render_andon_tui, render_compact},
    curriculum::{CurriculumConfig, CurriculumScheduler},
    DriftStatus, ErrorCategory, RuchyOracle,
};
use std::time::{Duration, Instant};

/// Configuration for the training loop
#[derive(Debug, Clone)]
pub struct TrainingLoopConfig {
    /// Maximum iterations before stopping (default: 50)
    pub max_iterations: usize,

    /// Target accuracy for convergence (default: 0.80)
    pub target_accuracy: f64,

    /// Enable automatic retraining on drift (default: true)
    pub auto_retrain: bool,

    /// Retrain threshold (samples since last train) (default: 100)
    pub retrain_threshold: usize,

    /// Display mode: compact, verbose, or silent
    pub display_mode: DisplayMode,

    /// Curriculum learning configuration
    pub curriculum: CurriculumConfig,

    /// Enable curriculum learning (default: true)
    pub curriculum_enabled: bool,
}

impl Default for TrainingLoopConfig {
    fn default() -> Self {
        Self {
            max_iterations: 50,
            target_accuracy: 0.80,
            auto_retrain: true,
            retrain_threshold: 100,
            display_mode: DisplayMode::Compact,
            curriculum: CurriculumConfig::default(),
            curriculum_enabled: true,
        }
    }
}

/// Display mode for training loop output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayMode {
    /// Full Andon TUI board
    Verbose,
    /// Single line status
    Compact,
    /// No output
    Silent,
}

/// Events emitted by the training loop
#[derive(Debug, Clone)]
pub enum TrainingEvent {
    /// Iteration completed normally
    IterationComplete {
        iteration: usize,
        accuracy: f64,
        drift_status: DriftStatus,
    },

    /// Target accuracy reached
    Converged {
        iteration: usize,
        accuracy: f64,
    },

    /// Retraining triggered (drift or threshold)
    RetrainingTriggered {
        reason: RetrainReason,
        samples: usize,
    },

    /// Retraining completed
    RetrainingComplete {
        accuracy_before: f64,
        accuracy_after: f64,
    },

    /// Curriculum level advanced
    CurriculumAdvanced {
        from: super::DifficultyLevel,
        to: super::DifficultyLevel,
    },

    /// Maximum iterations reached
    MaxIterationsReached {
        accuracy: f64,
    },

    /// Error occurred
    Error {
        message: String,
    },
}

/// Reason for retraining
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetrainReason {
    /// Drift detected
    Drift,
    /// Threshold reached
    Threshold,
    /// Manual trigger
    Manual,
}

/// Unified Training Loop
///
/// Manages the continuous learning process with visual feedback.
/// Implements default-on behavior per spec §13.2.
pub struct TrainingLoop {
    /// The Oracle being trained
    oracle: RuchyOracle,

    /// Configuration
    config: TrainingLoopConfig,

    /// Curriculum scheduler
    curriculum: CurriculumScheduler,

    /// Current iteration
    iteration: usize,

    /// Accuracy history for sparkline
    accuracy_history: Vec<f64>,

    /// Last training timestamp
    last_trained: Instant,

    /// Running state
    running: bool,

    /// Samples since last retrain
    samples_since_retrain: usize,
}

impl TrainingLoop {
    /// Create a new training loop with default configuration
    #[must_use]
    pub fn new(oracle: RuchyOracle) -> Self {
        Self::with_config(oracle, TrainingLoopConfig::default())
    }

    /// Create a training loop with custom configuration
    #[must_use]
    pub fn with_config(oracle: RuchyOracle, config: TrainingLoopConfig) -> Self {
        let curriculum = CurriculumScheduler::with_config(config.curriculum.clone());
        Self {
            oracle,
            config,
            curriculum,
            iteration: 0,
            accuracy_history: Vec::new(),
            last_trained: Instant::now(),
            running: false,
            samples_since_retrain: 0,
        }
    }

    /// Get configuration
    #[must_use]
    pub fn config(&self) -> &TrainingLoopConfig {
        &self.config
    }

    /// Get current iteration
    #[must_use]
    pub fn iteration(&self) -> usize {
        self.iteration
    }

    /// Check if loop is running
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get mutable reference to Oracle
    pub fn oracle_mut(&mut self) -> &mut RuchyOracle {
        &mut self.oracle
    }

    /// Get reference to Oracle
    #[must_use]
    pub fn oracle(&self) -> &RuchyOracle {
        &self.oracle
    }

    /// Get accuracy history
    #[must_use]
    pub fn accuracy_history(&self) -> &[f64] {
        &self.accuracy_history
    }

    /// Execute one iteration of the training loop
    pub fn step(&mut self) -> TrainingEvent {
        self.running = true;
        self.iteration += 1;

        // Get current accuracy (simplified - in production would evaluate on holdout set)
        let accuracy = self.oracle.metadata().training_accuracy;
        self.accuracy_history.push(accuracy);

        // Keep history bounded
        if self.accuracy_history.len() > 50 {
            self.accuracy_history.remove(0);
        }

        // Check drift status
        let drift_status = self.oracle.drift_status();

        // Check for retraining triggers
        if self.config.auto_retrain {
            if drift_status == DriftStatus::Drift {
                return self.trigger_retrain(RetrainReason::Drift);
            }

            if self.samples_since_retrain >= self.config.retrain_threshold {
                return self.trigger_retrain(RetrainReason::Threshold);
            }
        }

        // Check for convergence
        if accuracy >= self.config.target_accuracy {
            self.running = false;
            return TrainingEvent::Converged {
                iteration: self.iteration,
                accuracy,
            };
        }

        // Check max iterations
        if self.iteration >= self.config.max_iterations {
            self.running = false;
            return TrainingEvent::MaxIterationsReached { accuracy };
        }

        // Update curriculum if enabled
        if self.config.curriculum_enabled {
            let old_level = self.curriculum.current_level();
            self.curriculum.report_accuracy(accuracy);
            let new_level = self.curriculum.current_level();

            if old_level != new_level {
                return TrainingEvent::CurriculumAdvanced {
                    from: old_level,
                    to: new_level,
                };
            }
        }

        TrainingEvent::IterationComplete {
            iteration: self.iteration,
            accuracy,
            drift_status,
        }
    }

    /// Trigger retraining
    fn trigger_retrain(&mut self, _reason: RetrainReason) -> TrainingEvent {
        let _samples = self.samples_since_retrain;
        let accuracy_before = self.oracle.metadata().training_accuracy;

        // Attempt retrain
        match self.oracle.retrain() {
            Ok(()) => {
                self.samples_since_retrain = 0;
                self.last_trained = Instant::now();
                self.oracle.reset_drift_detector();

                let accuracy_after = self.oracle.metadata().training_accuracy;

                TrainingEvent::RetrainingComplete {
                    accuracy_before,
                    accuracy_after,
                }
            }
            Err(e) => TrainingEvent::Error {
                message: format!("Retraining failed: {e}"),
            },
        }
    }

    /// Record a classification result
    pub fn record_result(&mut self, predicted: ErrorCategory, actual: ErrorCategory) {
        self.oracle.record_result(predicted, actual);
        self.samples_since_retrain += 1;

        if self.config.curriculum_enabled {
            self.curriculum.record_prediction(predicted == actual);
        }
    }

    /// Render current status
    #[must_use]
    pub fn render(&self) -> String {
        match self.config.display_mode {
            DisplayMode::Verbose => self.render_verbose(),
            DisplayMode::Compact => self.render_compact(),
            DisplayMode::Silent => String::new(),
        }
    }

    /// Render verbose Andon TUI
    fn render_verbose(&self) -> String {
        let accuracy = self.oracle.metadata().training_accuracy;
        let delta = self.compute_accuracy_delta();
        let last_trained = self.format_last_trained();
        let model_size = 500; // Placeholder - would compute from serialized size

        render_andon_tui(
            self.iteration,
            self.config.max_iterations,
            accuracy,
            self.config.target_accuracy,
            delta,
            &last_trained,
            model_size,
            &self.accuracy_history,
            &self.oracle.drift_status(),
        )
    }

    /// Render compact status
    fn render_compact(&self) -> String {
        let accuracy = self.oracle.metadata().training_accuracy;
        let last_trained = self.format_last_trained_ago();
        let model_size = 500; // Placeholder

        render_compact(
            self.iteration,
            self.config.max_iterations,
            accuracy,
            model_size,
            &last_trained,
            &self.oracle.drift_status(),
        )
    }

    /// Compute accuracy delta from history
    fn compute_accuracy_delta(&self) -> f64 {
        if self.accuracy_history.len() < 2 {
            return 0.0;
        }
        let len = self.accuracy_history.len();
        self.accuracy_history[len - 1] - self.accuracy_history[len - 2]
    }

    /// Format last trained timestamp
    fn format_last_trained(&self) -> String {
        let elapsed = self.last_trained.elapsed();
        format!(
            "{} ({})",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            self.format_duration(elapsed)
        )
    }

    /// Format last trained as relative time
    fn format_last_trained_ago(&self) -> String {
        self.format_duration(self.last_trained.elapsed())
    }

    /// Format duration as human-readable string
    fn format_duration(&self, duration: Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{secs}s ago")
        } else if secs < 3600 {
            format!("{}m ago", secs / 60)
        } else if secs < 86400 {
            format!("{}h ago", secs / 3600)
        } else {
            format!("{}d ago", secs / 86400)
        }
    }

    /// Run the full training loop until convergence or max iterations
    pub fn run(&mut self) -> TrainingEvent {
        loop {
            let event = self.step();
            match &event {
                TrainingEvent::Converged { .. }
                | TrainingEvent::MaxIterationsReached { .. }
                | TrainingEvent::Error { .. } => {
                    self.running = false;
                    return event;
                }
                _ => {
                    // Print status if not silent
                    if self.config.display_mode != DisplayMode::Silent {
                        println!("{}", self.render());
                    }
                }
            }
        }
    }

    /// Stop the training loop
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Reset the training loop
    pub fn reset(&mut self) {
        self.iteration = 0;
        self.accuracy_history.clear();
        self.samples_since_retrain = 0;
        self.running = false;
        self.curriculum.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_training_loop_new() {
        let oracle = RuchyOracle::new();
        let loop_runner = TrainingLoop::new(oracle);
        assert!(!loop_runner.is_running());
        assert_eq!(loop_runner.iteration(), 0);
    }

    #[test]
    fn test_training_loop_with_config() {
        let config = TrainingLoopConfig {
            max_iterations: 100,
            target_accuracy: 0.90,
            ..Default::default()
        };
        let oracle = RuchyOracle::new();
        let loop_runner = TrainingLoop::with_config(oracle, config);
        assert_eq!(loop_runner.config().max_iterations, 100);
        assert!((loop_runner.config().target_accuracy - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn test_training_loop_step() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let mut loop_runner = TrainingLoop::new(oracle);
        let event = loop_runner.step();

        assert!(matches!(
            event,
            TrainingEvent::IterationComplete { .. } | TrainingEvent::Converged { .. }
        ));
        assert_eq!(loop_runner.iteration(), 1);
    }

    #[test]
    fn test_training_loop_convergence() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let config = TrainingLoopConfig {
            max_iterations: 10,
            target_accuracy: 0.50, // Low target for quick convergence
            auto_retrain: false,
            ..Default::default()
        };
        let mut loop_runner = TrainingLoop::with_config(oracle, config);

        // Run a few iterations
        let mut converged = false;
        for _ in 0..10 {
            match loop_runner.step() {
                TrainingEvent::Converged { accuracy, .. } => {
                    converged = true;
                    assert!(accuracy >= 0.50);
                    break;
                }
                TrainingEvent::MaxIterationsReached { .. } => break,
                _ => {}
            }
        }
        // Either converged or reached max iterations
        assert!(converged || loop_runner.iteration() >= 10);
    }

    #[test]
    fn test_training_loop_record_result() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let mut loop_runner = TrainingLoop::new(oracle);
        loop_runner.record_result(ErrorCategory::TypeMismatch, ErrorCategory::TypeMismatch);
        // No panic = success
    }

    #[test]
    fn test_training_loop_render_compact() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let config = TrainingLoopConfig {
            display_mode: DisplayMode::Compact,
            ..Default::default()
        };
        let loop_runner = TrainingLoop::with_config(oracle, config);
        let output = loop_runner.render();
        assert!(output.contains("Oracle"));
    }

    #[test]
    fn test_training_loop_render_silent() {
        let oracle = RuchyOracle::new();
        let config = TrainingLoopConfig {
            display_mode: DisplayMode::Silent,
            ..Default::default()
        };
        let loop_runner = TrainingLoop::with_config(oracle, config);
        let output = loop_runner.render();
        assert!(output.is_empty());
    }

    #[test]
    fn test_training_loop_reset() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let mut loop_runner = TrainingLoop::new(oracle);
        loop_runner.step();
        loop_runner.step();
        assert!(loop_runner.iteration() >= 2);

        loop_runner.reset();
        assert_eq!(loop_runner.iteration(), 0);
        assert!(loop_runner.accuracy_history().is_empty());
    }

    #[test]
    fn test_training_loop_config_default() {
        let config = TrainingLoopConfig::default();
        assert_eq!(config.max_iterations, 50);
        assert!((config.target_accuracy - 0.80).abs() < f64::EPSILON);
        assert!(config.auto_retrain);
    }

    #[test]
    fn test_display_mode_variants() {
        assert_eq!(DisplayMode::Verbose, DisplayMode::Verbose);
        assert_ne!(DisplayMode::Verbose, DisplayMode::Compact);
        assert_ne!(DisplayMode::Compact, DisplayMode::Silent);
    }

    #[test]
    fn test_retrain_reason_variants() {
        assert_eq!(RetrainReason::Drift, RetrainReason::Drift);
        assert_ne!(RetrainReason::Drift, RetrainReason::Threshold);
    }
}
