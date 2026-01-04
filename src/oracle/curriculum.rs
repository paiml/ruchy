//! Curriculum Learning Scheduler
//!
//! Implements progressive training difficulty for Oracle learning.
//! Based on curriculum learning principles from Bengio et al. (2009).
//!
//! # References
//! - [3] Bengio, Y., et al. (2009). "Curriculum Learning." ICML.
//! - Spec: docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md §3.3

use super::{Corpus, DifficultyLevel, Sample};

/// Configuration for curriculum learning
#[derive(Debug, Clone)]
pub struct CurriculumConfig {
    /// Accuracy threshold to advance to next level (default: 0.85)
    pub advance_threshold: f64,

    /// Samples per difficulty level before evaluation (default: 100)
    pub samples_per_level: usize,

    /// Minimum samples before allowing advancement (default: 10)
    pub min_samples_before_advance: usize,

    /// Enable automatic advancement (default: true)
    pub auto_advance: bool,
}

impl Default for CurriculumConfig {
    fn default() -> Self {
        Self {
            advance_threshold: 0.85,
            samples_per_level: 100,
            min_samples_before_advance: 10,
            auto_advance: true,
        }
    }
}

/// Curriculum Learning Scheduler
///
/// Manages progressive training difficulty following curriculum learning principles.
/// Starts with easy samples and advances to harder ones as accuracy improves.
#[derive(Debug)]
pub struct CurriculumScheduler {
    /// Current difficulty level
    current_level: DifficultyLevel,

    /// Configuration
    config: CurriculumConfig,

    /// Samples seen at current level
    samples_at_level: usize,

    /// Correct predictions at current level
    correct_at_level: usize,

    /// History of level advancements
    advancement_history: Vec<(DifficultyLevel, f64)>,
}

impl CurriculumScheduler {
    /// Create a new scheduler with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(CurriculumConfig::default())
    }

    /// Create a scheduler with custom configuration
    #[must_use]
    pub fn with_config(config: CurriculumConfig) -> Self {
        Self {
            current_level: DifficultyLevel::Easy,
            config,
            samples_at_level: 0,
            correct_at_level: 0,
            advancement_history: Vec::new(),
        }
    }

    /// Get current difficulty level
    #[must_use]
    pub fn current_level(&self) -> DifficultyLevel {
        self.current_level
    }

    /// Get configuration
    #[must_use]
    pub fn config(&self) -> &CurriculumConfig {
        &self.config
    }

    /// Report accuracy and potentially advance level
    pub fn report_accuracy(&mut self, accuracy: f64) {
        self.samples_at_level += 1;

        if accuracy >= self.config.advance_threshold {
            self.correct_at_level += 1;
        }

        // Check for advancement
        if self.config.auto_advance && self.should_advance() {
            self.advance();
        }
    }

    /// Record a single prediction result
    pub fn record_prediction(&mut self, correct: bool) {
        self.samples_at_level += 1;
        if correct {
            self.correct_at_level += 1;
        }

        // Check for advancement
        if self.config.auto_advance && self.should_advance() {
            self.advance();
        }
    }

    /// Check if advancement criteria are met
    #[must_use]
    pub fn should_advance(&self) -> bool {
        if self.current_level == DifficultyLevel::Expert {
            return false; // Already at max
        }

        if self.samples_at_level < self.config.min_samples_before_advance {
            return false; // Not enough samples
        }

        let accuracy = self.level_accuracy();
        accuracy >= self.config.advance_threshold
    }

    /// Get accuracy at current level
    #[must_use]
    pub fn level_accuracy(&self) -> f64 {
        if self.samples_at_level == 0 {
            0.0
        } else {
            self.correct_at_level as f64 / self.samples_at_level as f64
        }
    }

    /// Advance to next difficulty level
    pub fn advance(&mut self) {
        if self.current_level == DifficultyLevel::Expert {
            return;
        }

        let accuracy = self.level_accuracy();
        self.advancement_history
            .push((self.current_level, accuracy));

        self.current_level = self.current_level.next();
        self.samples_at_level = 0;
        self.correct_at_level = 0;
    }

    /// Get next training batch filtered by current difficulty
    #[must_use]
    pub fn next_batch<'a>(&self, corpus: &'a Corpus, limit: usize) -> Vec<&'a Sample> {
        let max_difficulty = self.current_level.score();

        corpus
            .samples()
            .iter()
            .filter(|s| s.difficulty <= max_difficulty)
            .take(limit)
            .collect()
    }

    /// Get samples at specific difficulty level
    #[must_use]
    pub fn samples_at_difficulty<'a>(
        &self,
        corpus: &'a Corpus,
        level: DifficultyLevel,
    ) -> Vec<&'a Sample> {
        let target_score = level.score();
        let tolerance = 0.125; // Half the gap between levels

        corpus
            .samples()
            .iter()
            .filter(|s| (s.difficulty - target_score).abs() <= tolerance)
            .collect()
    }

    /// Reset scheduler to initial state
    pub fn reset(&mut self) {
        self.current_level = DifficultyLevel::Easy;
        self.samples_at_level = 0;
        self.correct_at_level = 0;
        self.advancement_history.clear();
    }

    /// Get advancement history
    #[must_use]
    pub fn advancement_history(&self) -> &[(DifficultyLevel, f64)] {
        &self.advancement_history
    }

    /// Get samples processed at current level
    #[must_use]
    pub fn samples_processed(&self) -> usize {
        self.samples_at_level
    }
}

impl Default for CurriculumScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::{ErrorCategory, SampleSource};

    #[test]
    fn test_curriculum_scheduler_new() {
        let scheduler = CurriculumScheduler::new();
        assert_eq!(scheduler.current_level(), DifficultyLevel::Easy);
    }

    #[test]
    fn test_curriculum_scheduler_with_config() {
        let config = CurriculumConfig {
            advance_threshold: 0.90,
            samples_per_level: 50,
            ..Default::default()
        };
        let scheduler = CurriculumScheduler::with_config(config);
        assert_eq!(scheduler.config().advance_threshold, 0.90);
    }

    #[test]
    fn test_curriculum_advance_on_high_accuracy() {
        let config = CurriculumConfig {
            advance_threshold: 0.85,
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Easy);

        scheduler.report_accuracy(0.90);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Medium);
    }

    #[test]
    fn test_curriculum_no_advance_on_low_accuracy() {
        let config = CurriculumConfig {
            advance_threshold: 0.85,
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);
        scheduler.report_accuracy(0.70);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Easy);
    }

    #[test]
    fn test_curriculum_advance_through_all_levels() {
        let config = CurriculumConfig {
            advance_threshold: 0.85,
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        scheduler.report_accuracy(0.90);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Medium);

        scheduler.report_accuracy(0.90);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Hard);

        scheduler.report_accuracy(0.90);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Expert);

        // Should stay at Expert
        scheduler.report_accuracy(0.90);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Expert);
    }

    #[test]
    fn test_curriculum_next_batch_filters() {
        let mut corpus = Corpus::new();
        corpus.add(
            Sample::new("easy", Some("E0308".into()), ErrorCategory::TypeMismatch)
                .with_difficulty(0.25),
        );
        corpus.add(
            Sample::new("medium", Some("E0382".into()), ErrorCategory::BorrowChecker)
                .with_difficulty(0.50),
        );
        corpus.add(
            Sample::new("hard", Some("E0597".into()), ErrorCategory::LifetimeError)
                .with_difficulty(0.75),
        );

        let scheduler = CurriculumScheduler::new();
        let batch = scheduler.next_batch(&corpus, 10);

        // At Easy level, should only get Easy samples
        assert!(batch.iter().all(|s| s.difficulty <= 0.25));
    }

    #[test]
    fn test_curriculum_next_batch_limit() {
        let mut corpus = Corpus::new();
        for i in 0..100 {
            corpus.add(
                Sample::new(format!("error {i}"), None, ErrorCategory::TypeMismatch)
                    .with_difficulty(0.25)
                    .with_source(SampleSource::Synthetic),
            );
        }

        let scheduler = CurriculumScheduler::new();
        let batch = scheduler.next_batch(&corpus, 10);

        assert!(batch.len() <= 10);
    }

    #[test]
    fn test_curriculum_level_accuracy() {
        let config = CurriculumConfig {
            min_samples_before_advance: 100,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        scheduler.record_prediction(true);
        scheduler.record_prediction(true);
        scheduler.record_prediction(false);
        scheduler.record_prediction(true);

        assert!((scheduler.level_accuracy() - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_curriculum_reset() {
        let config = CurriculumConfig {
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        scheduler.report_accuracy(0.90);
        scheduler.report_accuracy(0.90);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Hard);

        scheduler.reset();
        assert_eq!(scheduler.current_level(), DifficultyLevel::Easy);
        assert_eq!(scheduler.samples_processed(), 0);
    }

    #[test]
    fn test_curriculum_config_default() {
        let config = CurriculumConfig::default();
        assert!((config.advance_threshold - 0.85).abs() < f64::EPSILON);
        assert_eq!(config.samples_per_level, 100);
    }

    // COVERAGE-95: Additional tests for complete coverage

    #[test]
    fn test_curriculum_scheduler_default() {
        let scheduler = CurriculumScheduler::default();
        assert_eq!(scheduler.current_level(), DifficultyLevel::Easy);
        assert_eq!(scheduler.samples_processed(), 0);
    }

    #[test]
    fn test_curriculum_config_fields() {
        let scheduler = CurriculumScheduler::new();
        let config = scheduler.config();
        assert!(config.auto_advance);
        assert_eq!(config.min_samples_before_advance, 10);
    }

    #[test]
    fn test_curriculum_samples_at_difficulty() {
        let mut corpus = Corpus::new();
        // Use completely different messages to avoid deduplication
        corpus.add(
            Sample::new("mismatched types: expected i32, found String", Some("E0308".into()), ErrorCategory::TypeMismatch)
                .with_difficulty(0.25),
        );
        corpus.add(
            Sample::new("mismatched types: expected bool, found char", Some("E0308".into()), ErrorCategory::TypeMismatch)
                .with_difficulty(0.25),
        );
        corpus.add(
            Sample::new("borrow of moved value", Some("E0382".into()), ErrorCategory::BorrowChecker)
                .with_difficulty(0.50),
        );

        let scheduler = CurriculumScheduler::new();
        let easy_samples = scheduler.samples_at_difficulty(&corpus, DifficultyLevel::Easy);
        assert_eq!(easy_samples.len(), 2);
    }

    #[test]
    fn test_curriculum_advancement_history() {
        let config = CurriculumConfig {
            advance_threshold: 0.85,
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        scheduler.report_accuracy(0.90);
        scheduler.report_accuracy(0.90);

        let history = scheduler.advancement_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].0, DifficultyLevel::Easy);
        assert_eq!(history[1].0, DifficultyLevel::Medium);
    }

    #[test]
    fn test_curriculum_should_advance_min_samples() {
        let config = CurriculumConfig {
            advance_threshold: 0.85,
            min_samples_before_advance: 10,
            auto_advance: false, // Disable auto-advance to test should_advance
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        // Not enough samples yet
        scheduler.record_prediction(true);
        assert!(!scheduler.should_advance());
    }

    #[test]
    fn test_curriculum_level_accuracy_zero_samples() {
        let scheduler = CurriculumScheduler::new();
        assert_eq!(scheduler.level_accuracy(), 0.0);
    }

    #[test]
    fn test_curriculum_advance_at_expert_noop() {
        let config = CurriculumConfig {
            advance_threshold: 0.85,
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        // Advance to Expert
        scheduler.report_accuracy(0.90); // Easy -> Medium
        scheduler.report_accuracy(0.90); // Medium -> Hard
        scheduler.report_accuracy(0.90); // Hard -> Expert
        assert_eq!(scheduler.current_level(), DifficultyLevel::Expert);

        // Try to advance again (should be noop)
        scheduler.advance();
        assert_eq!(scheduler.current_level(), DifficultyLevel::Expert);
    }

    #[test]
    fn test_curriculum_should_advance_at_expert() {
        let config = CurriculumConfig {
            advance_threshold: 0.85,
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        // Advance to Expert
        scheduler.report_accuracy(0.90);
        scheduler.report_accuracy(0.90);
        scheduler.report_accuracy(0.90);

        // Should not advance even with high accuracy
        assert!(!scheduler.should_advance());
    }

    #[test]
    fn test_curriculum_record_prediction_false() {
        let config = CurriculumConfig {
            min_samples_before_advance: 100,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        scheduler.record_prediction(false);
        scheduler.record_prediction(false);
        scheduler.record_prediction(false);

        assert_eq!(scheduler.level_accuracy(), 0.0);
        assert_eq!(scheduler.samples_processed(), 3);
    }

    #[test]
    fn test_curriculum_manual_advance() {
        let config = CurriculumConfig {
            auto_advance: false,
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        scheduler.record_prediction(true);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Easy);

        // Manual advance
        scheduler.advance();
        assert_eq!(scheduler.current_level(), DifficultyLevel::Medium);
    }

    #[test]
    fn test_curriculum_config_all_fields() {
        let config = CurriculumConfig {
            advance_threshold: 0.99,
            samples_per_level: 500,
            min_samples_before_advance: 50,
            auto_advance: false,
        };
        assert_eq!(config.advance_threshold, 0.99);
        assert_eq!(config.samples_per_level, 500);
        assert_eq!(config.min_samples_before_advance, 50);
        assert!(!config.auto_advance);
    }

    #[test]
    fn test_curriculum_reset_clears_history() {
        let config = CurriculumConfig {
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);

        scheduler.report_accuracy(0.90);
        assert!(!scheduler.advancement_history().is_empty());

        scheduler.reset();
        assert!(scheduler.advancement_history().is_empty());
    }
}
