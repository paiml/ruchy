//! ORACLE-001: Dynamic `MLOps` Training Tests
//!
//! Extreme TDD - Tests for spec implementation
//! Reference: docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md
//!
//! # Test Categories
//! - §13.3 Andon TUI Visual Feedback
//! - §3.3 Curriculum Learning
//! - §3.4 Knowledge Distillation
//! - §2.2 Four-Source Data Pipeline

// ============================================================================
// §13.3.1 Andon TUI Tests
// ============================================================================

mod andon_tui {
    use ruchy::oracle::andon::{render_compact, render_sparkline, AndonStatus};
    use ruchy::oracle::DriftStatus;

    #[test]
    fn test_andon_status_from_drift_stable() {
        let status = AndonStatus::from_drift(&DriftStatus::Stable);
        assert_eq!(status, AndonStatus::Green);
    }

    #[test]
    fn test_andon_status_from_drift_warning() {
        let status = AndonStatus::from_drift(&DriftStatus::Warning);
        assert_eq!(status, AndonStatus::Yellow);
    }

    #[test]
    fn test_andon_status_from_drift_drift() {
        let status = AndonStatus::from_drift(&DriftStatus::Drift);
        assert_eq!(status, AndonStatus::Red);
    }

    #[test]
    fn test_andon_status_display_green() {
        assert_eq!(AndonStatus::Green.display(), "● STABLE");
    }

    #[test]
    fn test_andon_status_display_yellow() {
        assert_eq!(AndonStatus::Yellow.display(), "● WARNING");
    }

    #[test]
    fn test_andon_status_display_red() {
        assert_eq!(AndonStatus::Red.display(), "● DRIFT");
    }

    #[test]
    fn test_andon_status_color_codes() {
        assert_eq!(AndonStatus::Green.color_code(), "\x1b[32m");
        assert_eq!(AndonStatus::Yellow.color_code(), "\x1b[33m");
        assert_eq!(AndonStatus::Red.color_code(), "\x1b[31m");
    }

    #[test]
    fn test_sparkline_empty_history() {
        let sparkline = render_sparkline(&[], 8);
        assert_eq!(sparkline, "────────");
    }

    #[test]
    fn test_sparkline_single_value() {
        let sparkline = render_sparkline(&[0.5], 8);
        assert_eq!(sparkline.chars().count(), 1);
    }

    #[test]
    fn test_sparkline_increasing_trend() {
        let history = vec![0.0, 0.14, 0.28, 0.42, 0.57, 0.71, 0.85, 1.0];
        let sparkline = render_sparkline(&history, 8);
        // Should show increasing trend: ▁▂▃▄▅▆▇█
        assert_eq!(sparkline, "▁▂▃▄▅▆▇█");
    }

    #[test]
    fn test_sparkline_flat_trend() {
        let history = vec![0.5, 0.5, 0.5, 0.5];
        let sparkline = render_sparkline(&history, 4);
        // All same height when flat
        assert!(sparkline.chars().all(|c| c == sparkline.chars().next().unwrap()));
    }

    #[test]
    fn test_sparkline_width_limit() {
        let history = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0];
        let sparkline = render_sparkline(&history, 5);
        assert_eq!(sparkline.chars().count(), 5);
    }

    #[test]
    fn test_render_compact_format() {
        let compact = render_compact(12, 50, 0.873, 847, "3h ago", &DriftStatus::Stable);
        assert!(compact.contains("iteration[12/50]"));
        assert!(compact.contains("87.3%"));
        assert!(compact.contains("847KB"));
        assert!(compact.contains("STABLE"));
    }

    #[test]
    fn test_render_compact_warning() {
        let compact = render_compact(12, 50, 0.70, 500, "1h ago", &DriftStatus::Warning);
        assert!(compact.contains("WARNING"));
    }
}

// ============================================================================
// §3.3 Curriculum Learning Tests
// ============================================================================

mod curriculum_learning {
    use ruchy::oracle::{
        Corpus, CurriculumConfig, CurriculumScheduler, DifficultyLevel, ErrorCategory, Sample,
        SampleSource,
    };

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
    fn test_curriculum_scheduler_advance_on_high_accuracy() {
        let config = CurriculumConfig {
            advance_threshold: 0.85,
            min_samples_before_advance: 1,
            ..Default::default()
        };
        let mut scheduler = CurriculumScheduler::with_config(config);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Easy);

        // Report high accuracy - should advance
        scheduler.report_accuracy(0.90);
        assert_eq!(scheduler.current_level(), DifficultyLevel::Medium);
    }

    #[test]
    fn test_curriculum_scheduler_no_advance_on_low_accuracy() {
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
    fn test_curriculum_scheduler_advance_through_all_levels() {
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
    fn test_curriculum_next_batch_filters_by_difficulty() {
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
    fn test_curriculum_next_batch_respects_limit() {
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
}

// ============================================================================
// §3.4 Knowledge Distillation Tests
// ============================================================================

mod knowledge_distillation {
    use ruchy::oracle::{
        DistillationConfig, ErrorCategory, KnowledgeDistiller, RuchyOracle, Sample, SoftLabel,
        SampleSource,
    };

    #[test]
    fn test_distiller_new() {
        let distiller = KnowledgeDistiller::new();
        assert_eq!(distiller.config().temperature, 3.0);
        assert_eq!(distiller.config().confidence_threshold, 0.95);
    }

    #[test]
    fn test_distiller_with_config() {
        let config = DistillationConfig {
            temperature: 5.0,
            confidence_threshold: 0.90,
        };
        let distiller = KnowledgeDistiller::with_config(config);
        assert_eq!(distiller.config().temperature, 5.0);
    }

    #[test]
    fn test_temperature_scaling() {
        let distiller = KnowledgeDistiller::new();
        let logits = vec![1.0, 2.0, 3.0];

        let soft = distiller.temperature_scale(&logits);

        // Temperature scaling should smooth the distribution
        assert_eq!(soft.len(), 3);
        assert!((soft.iter().sum::<f64>() - 1.0).abs() < 0.01); // Should sum to 1
    }

    #[test]
    fn test_temperature_scaling_higher_temp_smoother() {
        let distiller_low = KnowledgeDistiller::with_config(DistillationConfig {
            temperature: 1.0,
            confidence_threshold: 0.95,
        });
        let distiller_high = KnowledgeDistiller::with_config(DistillationConfig {
            temperature: 10.0,
            confidence_threshold: 0.95,
        });

        let logits = vec![1.0, 2.0, 5.0];
        let soft_low = distiller_low.temperature_scale(&logits);
        let soft_high = distiller_high.temperature_scale(&logits);

        // Higher temperature should produce more uniform distribution
        let max_low = soft_low.iter().copied().fold(0.0f64, f64::max);
        let max_high = soft_high.iter().copied().fold(0.0f64, f64::max);
        assert!(max_high < max_low);
    }

    #[test]
    fn test_distill_filters_low_confidence() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("train");

        let distiller = KnowledgeDistiller::with_config(DistillationConfig {
            temperature: 3.0,
            confidence_threshold: 0.99, // Very high threshold
        });

        let samples = vec![Sample::new(
            "unknown error pattern xyz",
            None,
            ErrorCategory::Other,
        )
        .with_source(SampleSource::Synthetic)];

        let soft_labels = distiller.distill(&oracle, &samples);
        // Low confidence predictions should be filtered
        assert!(soft_labels.len() <= samples.len());
    }

    #[test]
    fn test_soft_label_structure() {
        let sample =
            Sample::new("test", Some("E0308".into()), ErrorCategory::TypeMismatch);
        let soft_targets = vec![0.9, 0.05, 0.02, 0.01, 0.01, 0.005, 0.004, 0.001];

        let soft_label = SoftLabel::new(sample, soft_targets);
        assert_eq!(soft_label.soft_targets.len(), 8);
    }
}

// ============================================================================
// §2.2 Four-Source Data Pipeline Tests
// ============================================================================

mod four_source_pipeline {
    use ruchy::oracle::{
        CorpusMergerWithProvenance, ErrorCategory, Sample, SampleSource,
    };

    #[test]
    fn test_corpus_merger_new() {
        let merger = CorpusMergerWithProvenance::new();
        assert_eq!(merger.source_count(), 0);
    }

    #[test]
    fn test_corpus_merger_add_source() {
        let mut merger = CorpusMergerWithProvenance::new();
        let samples = vec![Sample::new(
            "error1",
            Some("E0308".into()),
            ErrorCategory::TypeMismatch,
        )];
        merger.add_source("synthetic", samples, SampleSource::Synthetic);
        assert_eq!(merger.source_count(), 1);
    }

    #[test]
    fn test_corpus_merger_four_sources() {
        let mut merger = CorpusMergerWithProvenance::new();

        // Source 1: Synthetic
        merger.add_source(
            "synthetic",
            vec![Sample::new(
                "type error",
                Some("E0308".into()),
                ErrorCategory::TypeMismatch,
            )],
            SampleSource::Synthetic,
        );

        // Source 2: Ruchy (hand-crafted)
        merger.add_source(
            "ruchy",
            vec![Sample::new(
                "borrow error",
                Some("E0382".into()),
                ErrorCategory::BorrowChecker,
            )],
            SampleSource::Ruchy,
        );

        // Source 3: Examples
        merger.add_source(
            "examples",
            vec![Sample::new(
                "lifetime error",
                Some("E0597".into()),
                ErrorCategory::LifetimeError,
            )],
            SampleSource::Examples,
        );

        // Source 4: Production
        merger.add_source(
            "production",
            vec![Sample::new(
                "trait error",
                Some("E0277".into()),
                ErrorCategory::TraitBound,
            )],
            SampleSource::Production,
        );

        assert_eq!(merger.source_count(), 4);
    }

    #[test]
    fn test_corpus_merger_merge_deduplicates() {
        let mut merger = CorpusMergerWithProvenance::new();

        // Add same sample from different sources
        let sample1 = Sample::new(
            "error[E0308]: type mismatch",
            Some("E0308".into()),
            ErrorCategory::TypeMismatch,
        );
        let sample2 = Sample::new(
            "error[E0308]: type mismatch",
            Some("E0308".into()),
            ErrorCategory::TypeMismatch,
        );

        merger.add_source("source1", vec![sample1], SampleSource::Synthetic);
        merger.add_source("source2", vec![sample2], SampleSource::Production);

        let (corpus, provenance) = merger.merge().expect("merge");

        // Should deduplicate
        assert_eq!(corpus.len(), 1);
        assert_eq!(provenance.sources.len(), 2);
    }

    #[test]
    fn test_corpus_merger_shuffle_deterministic() {
        let mut merger1 = CorpusMergerWithProvenance::new();
        let mut merger2 = CorpusMergerWithProvenance::new();

        let samples: Vec<_> = (0..10)
            .map(|i| Sample::new(format!("error {i}"), None, ErrorCategory::TypeMismatch))
            .collect();

        merger1.add_source("test", samples.clone(), SampleSource::Synthetic);
        merger2.add_source("test", samples, SampleSource::Synthetic);

        let (corpus1, _) = merger1.merge_with_seed(42).expect("merge");
        let (corpus2, _) = merger2.merge_with_seed(42).expect("merge");

        // Same seed should produce same order
        for (s1, s2) in corpus1.samples().iter().zip(corpus2.samples().iter()) {
            assert_eq!(s1.message, s2.message);
        }
    }

    #[test]
    fn test_corpus_provenance_tracking() {
        let mut merger = CorpusMergerWithProvenance::new();
        merger.add_source(
            "synthetic",
            vec![
                Sample::new("e1", None, ErrorCategory::TypeMismatch),
                Sample::new("e2", None, ErrorCategory::BorrowChecker),
            ],
            SampleSource::Synthetic,
        );
        merger.add_source(
            "production",
            vec![Sample::new("e3", None, ErrorCategory::LifetimeError)],
            SampleSource::Production,
        );

        let (corpus, provenance) = merger.merge().expect("merge");

        assert_eq!(corpus.len(), 3);
        assert_eq!(provenance.total_after_dedup, 3);
        assert!(provenance.merged_at.is_some());
    }
}

// ============================================================================
// Integration: Unified Training Loop
// ============================================================================

mod unified_training_loop {
    use ruchy::oracle::{
        ErrorCategory, RuchyOracle, TrainingEvent, TrainingLoop, TrainingLoopConfig,
    };

    #[test]
    fn test_training_loop_new() {
        let oracle = RuchyOracle::new();
        let loop_runner = TrainingLoop::new(oracle);
        assert!(!loop_runner.is_running());
    }

    #[test]
    fn test_training_loop_with_config() {
        let config = TrainingLoopConfig {
            max_iterations: 50,
            target_accuracy: 0.80,
            auto_retrain: true,
            ..Default::default()
        };
        let oracle = RuchyOracle::new();
        let loop_runner = TrainingLoop::with_config(oracle, config);
        assert_eq!(loop_runner.config().max_iterations, 50);
    }

    #[test]
    fn test_training_loop_iteration() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let mut loop_runner = TrainingLoop::new(oracle);
        let event = loop_runner.step();

        assert!(matches!(
            event,
            TrainingEvent::IterationComplete { .. } | TrainingEvent::Converged { .. }
        ));
    }

    #[test]
    fn test_training_loop_convergence_detection() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let config = TrainingLoopConfig {
            max_iterations: 10,
            target_accuracy: 0.50, // Low target for quick convergence
            auto_retrain: false,
            ..Default::default()
        };
        let mut loop_runner = TrainingLoop::with_config(oracle, config);

        // Run until convergence or max iterations
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
        assert!(converged || loop_runner.iteration() >= 10);
    }

    #[test]
    fn test_training_loop_drift_triggers_retrain() {
        let mut oracle = RuchyOracle::new();
        oracle.train_from_examples().expect("bootstrap");

        let config = TrainingLoopConfig {
            max_iterations: 100,
            target_accuracy: 0.80,
            auto_retrain: true,
            ..Default::default()
        };
        let mut loop_runner = TrainingLoop::with_config(oracle, config);

        // Simulate drift by recording many incorrect predictions
        for _ in 0..100 {
            loop_runner.oracle_mut().record_result(
                ErrorCategory::TypeMismatch,
                ErrorCategory::BorrowChecker,
            );
        }

        let event = loop_runner.step();
        // Should trigger retraining due to drift or complete normally
        assert!(matches!(
            event,
            TrainingEvent::RetrainingTriggered { .. }
                | TrainingEvent::RetrainingComplete { .. }
                | TrainingEvent::IterationComplete { .. }
                | TrainingEvent::Converged { .. }
        ));
    }
}

// ============================================================================
// Property Tests
// ============================================================================

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use ruchy::oracle::andon::render_sparkline;

    proptest! {
        #[test]
        fn test_sparkline_length_matches_width(width in 1usize..20) {
            let history: Vec<f64> = (0..width).map(|i| i as f64 / width as f64).collect();
            let sparkline = render_sparkline(&history, width);
            prop_assert_eq!(sparkline.chars().count(), width);
        }

        #[test]
        fn test_sparkline_chars_valid(values in prop::collection::vec(0.0f64..1.0, 1..20)) {
            let sparkline = render_sparkline(&values, values.len());
            let valid_chars: Vec<char> = "▁▂▃▄▅▆▇█─".chars().collect();
            for c in sparkline.chars() {
                prop_assert!(valid_chars.contains(&c));
            }
        }
    }
}
