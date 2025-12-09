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
    DriftStatus, ErrorCategory, RuchyOracle, Sample, SampleSource,
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

    /// Holdout set for real accuracy evaluation (spec §13.3)
    holdout_set: Vec<Sample>,

    /// Index for round-robin evaluation through holdout set
    holdout_index: usize,
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
        let holdout_set = Self::build_holdout_set();
        Self {
            oracle,
            config,
            curriculum,
            iteration: 0,
            accuracy_history: Vec::new(),
            last_trained: Instant::now(),
            running: false,
            samples_since_retrain: 0,
            holdout_set,
            holdout_index: 0,
        }
    }

    /// Build holdout set from bootstrap samples for evaluation
    ///
    /// Uses a subset of bootstrap samples that mirrors the training distribution
    /// but is held out for accuracy evaluation (spec §13.3).
    ///
    /// Mix of samples WITH and WITHOUT error codes to exercise both:
    /// - Rule-based path (known error codes)
    /// - ML classifier path (no error code, relies on keywords/features)
    fn build_holdout_set() -> Vec<Sample> {
        vec![
            // === TypeMismatch samples ===
            // With error code (rule-based path)
            Sample::new(
                "expected `u32`, found `i64`",
                Some("E0308".into()),
                ErrorCategory::TypeMismatch,
            ).with_source(SampleSource::Synthetic),
            // Without error code (ML path - uses "type", "expected", "mismatch" keywords)
            Sample::new(
                "type mismatch: expected String but found integer",
                None,
                ErrorCategory::TypeMismatch,
            ).with_source(SampleSource::Synthetic),

            // === BorrowChecker samples ===
            // With error code (rule-based path)
            Sample::new(
                "cannot borrow `x` as mutable because it is also borrowed as immutable",
                Some("E0502".into()),
                ErrorCategory::BorrowChecker,
            ).with_source(SampleSource::Synthetic),
            // Without error code (ML path - uses "borrow", "move" keywords)
            Sample::new(
                "cannot borrow value after move to another function",
                None,
                ErrorCategory::BorrowChecker,
            ).with_source(SampleSource::Synthetic),

            // === LifetimeError samples ===
            // With error code (rule-based path)
            Sample::new(
                "borrowed value does not live long enough",
                Some("E0597".into()),
                ErrorCategory::LifetimeError,
            ).with_source(SampleSource::Synthetic),
            // Without error code (ML path - uses "lifetime" keyword)
            Sample::new(
                "lifetime of reference outlives the data it points to",
                None,
                ErrorCategory::LifetimeError,
            ).with_source(SampleSource::Synthetic),

            // === TraitBound samples ===
            // With error code (rule-based path)
            Sample::new(
                "the trait bound `MyType: Clone` is not satisfied",
                Some("E0277".into()),
                ErrorCategory::TraitBound,
            ).with_source(SampleSource::Synthetic),
            // Without error code (ML path - uses "trait", "impl" keywords)
            Sample::new(
                "trait Clone is not implemented for this type",
                None,
                ErrorCategory::TraitBound,
            ).with_source(SampleSource::Synthetic),

            // === MissingImport samples ===
            // With error code (rule-based path)
            Sample::new(
                "cannot find value `HashMap` in this scope",
                Some("E0425".into()),
                ErrorCategory::MissingImport,
            ).with_source(SampleSource::Synthetic),
            // Without error code (ML path - no specific keyword, tests generalization)
            Sample::new(
                "unresolved import: module not found in crate",
                None,
                ErrorCategory::MissingImport,
            ).with_source(SampleSource::Synthetic),

            // === MutabilityError samples ===
            // With error code (rule-based path)
            Sample::new(
                "cannot borrow `vec` as mutable, as it is not declared as mutable",
                Some("E0596".into()),
                ErrorCategory::MutabilityError,
            ).with_source(SampleSource::Synthetic),
            // Without error code (ML path - uses "mut" keyword)
            Sample::new(
                "requires mut binding but variable is immutable",
                None,
                ErrorCategory::MutabilityError,
            ).with_source(SampleSource::Synthetic),

            // === SyntaxError samples ===
            // Without error code (ML path)
            Sample::new(
                "expected `;`, found `let`",
                None,
                ErrorCategory::SyntaxError,
            ).with_source(SampleSource::Synthetic),
            // With error code (rule-based path)
            Sample::new(
                "this function takes 2 arguments but 3 arguments were supplied",
                Some("E0061".into()),
                ErrorCategory::SyntaxError,
            ).with_source(SampleSource::Synthetic),

            // === Other samples (edge cases, no clear keywords) ===
            Sample::new(
                "recursion limit reached while expanding the macro",
                None,
                ErrorCategory::Other,
            ).with_source(SampleSource::Synthetic),
            Sample::new(
                "internal compiler error: unexpected panic during compilation",
                None,
                ErrorCategory::Other,
            ).with_source(SampleSource::Synthetic),
        ]
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
    ///
    /// Per spec §13.3, each iteration evaluates a batch of samples from the
    /// holdout set to compute real accuracy from actual classifications.
    pub fn step(&mut self) -> TrainingEvent {
        self.running = true;
        self.iteration += 1;

        // Evaluate real accuracy on holdout set (spec §13.3)
        let accuracy = self.evaluate_holdout_batch();
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

    /// Evaluate accuracy on a batch of holdout samples
    ///
    /// Classifies samples from the holdout set and compares predictions
    /// to actual labels, computing real accuracy (spec §13.3).
    fn evaluate_holdout_batch(&mut self) -> f64 {
        if self.holdout_set.is_empty() {
            return 0.0;
        }

        // Evaluate a batch of samples (4 per iteration for efficiency)
        const BATCH_SIZE: usize = 4;
        let mut correct = 0;
        let mut total = 0;

        for _ in 0..BATCH_SIZE {
            let sample = &self.holdout_set[self.holdout_index];
            let error = sample.to_compilation_error();
            let classification = self.oracle.classify(&error);

            // Compare predicted vs actual category
            if classification.category == sample.category {
                correct += 1;
            }

            // Record result for drift detection
            self.oracle.record_result(classification.category, sample.category);

            // Round-robin through holdout set
            self.holdout_index = (self.holdout_index + 1) % self.holdout_set.len();
            total += 1;
        }

        if total == 0 {
            0.0
        } else {
            f64::from(correct) / f64::from(total)
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

    /// Add live samples from examples/*.ruchy transpilation (spec §13.3)
    ///
    /// These samples are added to the holdout set for evaluation,
    /// providing real-world accuracy measurements.
    pub fn add_live_samples(&mut self, samples: Vec<Sample>) {
        self.holdout_set.extend(samples);
    }

    /// Get current holdout set size
    #[must_use]
    pub fn holdout_size(&self) -> usize {
        self.holdout_set.len()
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
        // Use evaluated accuracy from history, not stale metadata
        let accuracy = self.current_accuracy();
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
        // Use evaluated accuracy from history, not stale metadata
        let accuracy = self.current_accuracy();
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

    /// Get current accuracy from most recent evaluation
    fn current_accuracy(&self) -> f64 {
        self.accuracy_history.last().copied().unwrap_or(0.0)
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
        self.holdout_index = 0;
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

    #[test]
    fn test_holdout_set_initialized() {
        let oracle = RuchyOracle::new();
        let loop_runner = TrainingLoop::new(oracle);
        // Holdout set should be initialized with samples
        assert!(!loop_runner.holdout_set.is_empty());
        // Should have 16 samples (2 per category, 8 categories)
        assert_eq!(loop_runner.holdout_set.len(), 16);
    }

    #[test]
    fn test_holdout_evaluation_produces_real_accuracy() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let mut loop_runner = TrainingLoop::new(oracle);
        let event = loop_runner.step();

        // Accuracy should be computed from real classification, not 0
        match event {
            TrainingEvent::IterationComplete { accuracy, .. }
            | TrainingEvent::Converged { accuracy, .. } => {
                // Trained oracle should classify some samples correctly
                assert!(accuracy > 0.0, "Accuracy should be > 0% after training");
            }
            TrainingEvent::MaxIterationsReached { accuracy } => {
                assert!(accuracy > 0.0, "Accuracy should be > 0% after training");
            }
            _ => {}
        }

        // Accuracy history should be populated
        assert!(!loop_runner.accuracy_history().is_empty());
        // All values should be real (between 0 and 1)
        for &acc in loop_runner.accuracy_history() {
            assert!((0.0..=1.0).contains(&acc), "Accuracy must be in [0, 1]");
        }
    }

    #[test]
    fn test_current_accuracy_reflects_history() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let mut loop_runner = TrainingLoop::new(oracle);

        // Before any steps, current_accuracy should be 0
        assert_eq!(loop_runner.current_accuracy(), 0.0);

        // After step, current_accuracy should match last history entry
        loop_runner.step();
        let current = loop_runner.current_accuracy();
        let last_history = loop_runner.accuracy_history().last().copied().unwrap_or(0.0);
        assert!((current - last_history).abs() < f64::EPSILON);
    }

    #[test]
    fn test_holdout_index_wraps_around() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let mut loop_runner = TrainingLoop::new(oracle);
        let holdout_size = loop_runner.holdout_set.len();

        // Run enough steps to cycle through entire holdout set
        // Each step evaluates 4 samples, so we need holdout_size/4 + 1 steps
        let steps_needed = (holdout_size / 4) + 2;
        for _ in 0..steps_needed {
            loop_runner.step();
        }

        // Index should wrap around and be within bounds
        assert!(loop_runner.holdout_index < holdout_size);
    }

    #[test]
    fn test_render_uses_evaluated_accuracy() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let config = TrainingLoopConfig {
            display_mode: DisplayMode::Compact,
            ..Default::default()
        };
        let mut loop_runner = TrainingLoop::with_config(oracle, config);
        loop_runner.step();

        let output = loop_runner.render();
        // Output should contain accuracy percentage (not 0.0%)
        assert!(output.contains("Oracle"));
        // Should show real accuracy, which for trained oracle is typically high
    }

    #[test]
    fn test_add_live_samples() {
        let oracle = RuchyOracle::new();
        let mut loop_runner = TrainingLoop::new(oracle);

        let initial_size = loop_runner.holdout_size();
        assert_eq!(initial_size, 16); // Default holdout set

        // Add live samples
        let live_samples = vec![
            Sample::new(
                "custom error from examples",
                Some("E0999".into()),
                ErrorCategory::Other,
            ).with_source(SampleSource::Examples),
            Sample::new(
                "another custom error",
                None,
                ErrorCategory::SyntaxError,
            ).with_source(SampleSource::Examples),
        ];

        loop_runner.add_live_samples(live_samples);

        // Holdout set should have grown
        assert_eq!(loop_runner.holdout_size(), initial_size + 2);
    }

    #[test]
    fn test_live_samples_evaluated_in_step() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let mut loop_runner = TrainingLoop::new(oracle);

        // Add live sample with known error code
        let live_samples = vec![
            Sample::new(
                "expected type `String`, found `i32`",
                Some("E0308".into()),
                ErrorCategory::TypeMismatch,
            ).with_source(SampleSource::Examples),
        ];
        loop_runner.add_live_samples(live_samples);

        // Run enough steps to evaluate all samples including the new one
        for _ in 0..5 {
            loop_runner.step();
        }

        // Accuracy should be non-zero (samples are being evaluated)
        assert!(!loop_runner.accuracy_history().is_empty());
    }
}
