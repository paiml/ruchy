//! ORACLE-002: Auto-train on transpilation errors
//!
//! Extreme TDD - Tests for automatic corpus collection and retraining
//!
//! # Spec Reference
//! - docs/specifications/dynamic-mlops-training-ruchy-oracle-spec.md §2.2

use ruchy::oracle::{Corpus, ErrorCategory, RuchyOracle, Sample, SampleSource};

// ============================================================================
// Corpus collection tests
// ============================================================================

#[test]
fn test_corpus_new_empty() {
    let corpus = Corpus::new();
    assert_eq!(corpus.len(), 0);
    assert!(corpus.is_empty());
}

#[test]
fn test_corpus_add_sample() {
    let mut corpus = Corpus::new();
    let sample = Sample::new(
        "error[E0308]: mismatched types",
        Some("E0308".to_string()),
        ErrorCategory::TypeMismatch,
    )
    .with_source(SampleSource::Production);
    corpus.add(sample);
    assert_eq!(corpus.len(), 1);
}

#[test]
fn test_corpus_deduplicates_by_hash() {
    let mut corpus = Corpus::new();

    // Add same error twice
    let sample1 = Sample::new(
        "error[E0308]: mismatched types",
        Some("E0308".to_string()),
        ErrorCategory::TypeMismatch,
    )
    .with_source(SampleSource::Production);
    let sample2 = Sample::new(
        "error[E0308]: mismatched types",
        Some("E0308".to_string()),
        ErrorCategory::TypeMismatch,
    )
    .with_source(SampleSource::Production);

    corpus.add(sample1);
    corpus.add(sample2);

    // Should deduplicate (features hash to same value)
    assert_eq!(corpus.len(), 1);
}

#[test]
fn test_corpus_to_training_data() {
    let mut corpus = Corpus::new();
    corpus.add(
        Sample::new(
            "error[E0308]: mismatched types",
            Some("E0308".to_string()),
            ErrorCategory::TypeMismatch,
        )
        .with_source(SampleSource::Synthetic),
    );
    corpus.add(
        Sample::new(
            "error[E0382]: borrow of moved value",
            Some("E0382".to_string()),
            ErrorCategory::BorrowChecker,
        )
        .with_source(SampleSource::Synthetic),
    );

    let (features, labels) = corpus.to_training_data();
    assert_eq!(features.len(), 2);
    assert_eq!(labels.len(), 2);
}

#[test]
fn test_corpus_filter_by_source() {
    let mut corpus = Corpus::new();
    corpus.add(
        Sample::new(
            "synthetic error 1",
            Some("E0308".to_string()),
            ErrorCategory::TypeMismatch,
        )
        .with_source(SampleSource::Synthetic),
    );
    corpus.add(
        Sample::new(
            "production error 1",
            Some("E0382".to_string()),
            ErrorCategory::BorrowChecker,
        )
        .with_source(SampleSource::Production),
    );

    let synthetic = corpus.filter_by_source(SampleSource::Synthetic);
    assert_eq!(synthetic.len(), 1);

    let production = corpus.filter_by_source(SampleSource::Production);
    assert_eq!(production.len(), 1);
}

// ============================================================================
// Auto-retrain threshold tests
// ============================================================================

#[test]
fn test_oracle_should_retrain_below_threshold() {
    let mut oracle = RuchyOracle::new();
    oracle.train_from_examples().expect("bootstrap");

    // Add fewer samples than threshold (default: 100)
    for i in 0..50 {
        oracle.record_error(&format!("error {i}"), ErrorCategory::TypeMismatch);
    }

    assert!(!oracle.should_retrain());
}

#[test]
fn test_oracle_should_retrain_above_threshold() {
    let mut oracle = RuchyOracle::new();
    oracle.train_from_examples().expect("bootstrap");

    // Add more samples than threshold
    for i in 0..150 {
        oracle.record_error(&format!("error {i}"), ErrorCategory::TypeMismatch);
    }

    assert!(oracle.should_retrain());
}

#[test]
fn test_oracle_auto_retrain_improves_accuracy() {
    let mut oracle = RuchyOracle::new();
    oracle.train_from_examples().expect("bootstrap");

    let initial_accuracy = oracle.metadata().training_accuracy;

    // Add diverse training samples
    oracle.record_error(
        "error[E0308]: mismatched types",
        ErrorCategory::TypeMismatch,
    );
    oracle.record_error(
        "error[E0382]: borrow of moved value",
        ErrorCategory::BorrowChecker,
    );
    oracle.record_error(
        "error[E0277]: trait bound not satisfied",
        ErrorCategory::TraitBound,
    );

    // Trigger retrain
    oracle.retrain().expect("retrain");

    // Accuracy should be maintained or improved (allow 10% margin)
    assert!(oracle.metadata().training_accuracy >= initial_accuracy * 0.9);
}

// ============================================================================
// Integration with transpile
// ============================================================================

#[test]
fn test_oracle_collect_from_rustc_output() {
    let rustc_stderr = r#"
error[E0308]: mismatched types
  --> src/main.rs:5:12
   |
5  |     let x: i32 = "hello";
   |            ---   ^^^^^^^ expected `i32`, found `&str`
   |            |
   |            expected due to this

error: aborting due to previous error
"#;

    let samples = RuchyOracle::parse_rustc_errors(rustc_stderr);
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].category, ErrorCategory::TypeMismatch);
}

#[test]
fn test_oracle_collect_multiple_errors() {
    let rustc_stderr = r"
error[E0308]: mismatched types
  --> src/main.rs:5:12

error[E0382]: borrow of moved value: `x`
  --> src/main.rs:10:5

error[E0277]: the trait bound `String: Copy` is not satisfied
  --> src/main.rs:15:10
";

    let samples = RuchyOracle::parse_rustc_errors(rustc_stderr);
    assert_eq!(samples.len(), 3);
}

#[test]
fn test_oracle_ignores_warnings() {
    let rustc_stderr = r"
warning: unused variable: `x`
  --> src/main.rs:5:9

error[E0308]: mismatched types
  --> src/main.rs:10:12
";

    let samples = RuchyOracle::parse_rustc_errors(rustc_stderr);
    assert_eq!(samples.len(), 1); // Only the error, not warning
}

// ============================================================================
// Drift detection integration
// ============================================================================

#[test]
fn test_oracle_drift_detection_api_works() {
    use ruchy::oracle::DriftStatus;

    let mut oracle = RuchyOracle::new();
    oracle.train_from_examples().expect("bootstrap");

    // Record some predictions to exercise drift detection API
    for _ in 0..50 {
        oracle.record_result(ErrorCategory::TypeMismatch, ErrorCategory::TypeMismatch);
    }

    // Verify drift_detected() returns a valid result (doesn't panic)
    let _detected = oracle.drift_detected();

    // Verify drift_status() returns a valid DriftStatus
    let status = oracle.drift_status();
    assert!(matches!(
        status,
        DriftStatus::Stable | DriftStatus::Warning | DriftStatus::Drift
    ));
}

#[test]
fn test_oracle_no_drift_with_correct_predictions() {
    let mut oracle = RuchyOracle::new();
    oracle.train_from_examples().expect("bootstrap");

    // Simulate correct predictions
    for _ in 0..100 {
        oracle.record_result(ErrorCategory::TypeMismatch, ErrorCategory::TypeMismatch);
    }

    assert!(!oracle.drift_detected());
}

// ============================================================================
// Synthetic data generation
// ============================================================================

#[test]
fn test_generate_synthetic_samples() {
    let samples = RuchyOracle::generate_synthetic_samples(100);
    assert_eq!(samples.len(), 100);

    // Should have diverse categories
    let categories: std::collections::HashSet<_> = samples.iter().map(|s| s.category).collect();
    assert!(categories.len() >= 4); // At least 4 different categories
}

#[test]
fn test_synthetic_samples_balanced() {
    let samples = RuchyOracle::generate_synthetic_samples(800);

    // Count per category
    let mut counts = std::collections::HashMap::new();
    for sample in &samples {
        *counts.entry(sample.category).or_insert(0) += 1;
    }

    // Each category should have roughly equal samples (within 50%)
    let avg = 800 / 8; // 8 categories
    for (_cat, count) in counts {
        assert!(count >= avg / 2, "Category underrepresented");
        assert!(count <= avg * 2, "Category overrepresented");
    }
}
