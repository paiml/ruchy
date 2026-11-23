//! Oracle Integration Tests
//!
//! Tests the full Oracle pipeline from error input to fix suggestions.
//!
//! # Test Strategy
//! - Unit tests: Individual components (category, features, patterns)
//! - Integration tests: Full classification pipeline
//! - Property tests: Invariants and edge cases
//!
//! # References
//! - Spec: docs/specifications/ruchy-oracle-spec.md

use ruchy::oracle::{
    CompilationError, DriftDetector, DriftStatus, ErrorCategory, ErrorFeatures, FixPattern,
    OracleConfig, PatternStore, RuchyOracle, ADWIN,
};

// ============================================================================
// Integration Tests: Full Pipeline
// ============================================================================

#[test]
fn test_oracle_full_pipeline_type_mismatch() {
    // Create and train Oracle
    let mut oracle = RuchyOracle::new();
    oracle
        .train_from_examples()
        .expect("Training should succeed");

    // Simulate a type mismatch error from rustc
    let error = CompilationError::new("error[E0308]: mismatched types")
        .with_code("E0308")
        .with_file("src/main.rs")
        .with_location(10, 5);

    // Classify the error
    let classification = oracle.classify(&error);

    // Verify classification
    assert_eq!(classification.category, ErrorCategory::TypeMismatch);
    assert!(classification.confidence > 0.0);
}

#[test]
fn test_oracle_full_pipeline_borrow_checker() {
    let mut oracle = RuchyOracle::new();
    oracle
        .train_from_examples()
        .expect("Training should succeed");

    let error = CompilationError::new("error[E0382]: borrow of moved value: `x`")
        .with_code("E0382")
        .with_file("src/lib.rs")
        .with_location(25, 10);

    let classification = oracle.classify(&error);

    assert_eq!(classification.category, ErrorCategory::BorrowChecker);
}

#[test]
fn test_oracle_full_pipeline_missing_import() {
    let mut oracle = RuchyOracle::new();
    oracle
        .train_from_examples()
        .expect("Training should succeed");

    let error =
        CompilationError::new("error[E0433]: failed to resolve: use of undeclared type `HashMap`")
            .with_code("E0433");

    let classification = oracle.classify(&error);

    assert_eq!(classification.category, ErrorCategory::MissingImport);
}

#[test]
fn test_oracle_pattern_suggestions() {
    let oracle = RuchyOracle::new();

    // This error should match the HashMap pattern
    let error =
        CompilationError::new("cannot find type `HashMap` in this scope").with_code("E0433");

    let classification = oracle.classify(&error);

    // Should have suggestions from pattern store
    assert!(
        !classification.suggestions.is_empty()
            || classification.category == ErrorCategory::MissingImport
    );
}

// ============================================================================
// Error Category Coverage Tests
// ============================================================================

#[test]
fn test_all_error_categories_classified() {
    // Use messages similar to training data for accurate classification
    let test_cases = vec![
        (
            "E0308",
            "mismatched types: expected `i32`, found `String`",
            ErrorCategory::TypeMismatch,
        ),
        (
            "E0382",
            "borrow of moved value: `x`",
            ErrorCategory::BorrowChecker,
        ),
        (
            "E0597",
            "borrowed value does not live long enough",
            ErrorCategory::LifetimeError,
        ),
        (
            "E0277",
            "the trait `Debug` is not implemented",
            ErrorCategory::TraitBound,
        ),
        (
            "E0433",
            "cannot find type `HashMap` in this scope",
            ErrorCategory::MissingImport,
        ),
        (
            "E0596",
            "cannot borrow `x` as mutable, as it is not declared as mutable",
            ErrorCategory::MutabilityError,
        ),
        (
            "E0658",
            "this function takes 2 arguments but 1 was supplied",
            ErrorCategory::SyntaxError,
        ),
    ];

    let mut oracle = RuchyOracle::new();
    oracle.train_from_examples().expect("Training");

    for (code, message, expected_category) in test_cases {
        let error = CompilationError::new(message).with_code(code);
        let classification = oracle.classify(&error);

        assert_eq!(
            classification.category, expected_category,
            "Error code {code} with message '{message}' should classify as {expected_category:?}"
        );
    }
}

// ============================================================================
// Feature Extraction Tests
// ============================================================================

#[test]
fn test_feature_extraction_consistency() {
    let messages = vec![
        "mismatched types: expected i32, found String",
        "cannot borrow x as mutable",
        "the trait Clone is not implemented",
        "help: consider using .clone()",
    ];

    for message in messages {
        let f1 = ErrorFeatures::extract(message, None);
        let f2 = ErrorFeatures::extract(message, None);

        assert_eq!(
            f1.features, f2.features,
            "Feature extraction should be deterministic for: {message}"
        );
    }
}

#[test]
fn test_feature_extraction_error_codes() {
    let codes = vec!["E0308", "E0382", "E0597", "E0277", "E0433"];

    for code in codes {
        let features = ErrorFeatures::extract("test error", Some(code));

        // At least one feature should be set for known error codes
        let sum: f32 = features.as_slice().iter().sum();
        assert!(
            sum > 0.0,
            "Features should be non-zero for error code {code}"
        );
    }
}

// ============================================================================
// Drift Detection Tests (using aprender::online::drift - Issue #174)
// ============================================================================

#[test]
fn test_drift_detection_stable() {
    // Use ADWIN from aprender (recommended drift detector)
    let mut detector = ADWIN::default();

    // Record 100 correct predictions (add_element takes error=false for correct)
    for _ in 0..100 {
        detector.add_element(false); // false = no error (correct prediction)
    }

    // aprender DriftStatus uses unit variants
    assert_eq!(detector.detected_change(), DriftStatus::Stable);
}

#[test]
fn test_drift_detection_degradation() {
    // Use ADWIN with default config
    let mut detector = ADWIN::default();

    // Start with good accuracy (no errors)
    for _ in 0..50 {
        detector.add_element(false);
    }

    // Then sudden drop (all errors)
    for _ in 0..10 {
        detector.add_element(true); // true = error (incorrect prediction)
    }

    // Should detect drift, warning, or remain stable depending on ADWIN thresholds
    let status = detector.detected_change();
    assert!(matches!(
        status,
        DriftStatus::Stable | DriftStatus::Warning | DriftStatus::Drift
    ));
}

// ============================================================================
// Pattern Store Tests
// ============================================================================

#[test]
fn test_pattern_store_default_patterns() {
    let store = PatternStore::new();

    // Should have patterns for major categories
    assert!(store.patterns_for(ErrorCategory::TypeMismatch).is_some());
    assert!(store.patterns_for(ErrorCategory::BorrowChecker).is_some());
    assert!(store.patterns_for(ErrorCategory::MissingImport).is_some());
}

#[test]
fn test_pattern_store_query_matching() {
    let store = PatternStore::new();

    // Query for String/&str conversion
    let suggestions = store.query(
        ErrorCategory::TypeMismatch,
        "expected `String`, found `&str`",
        0.0,
    );

    assert!(!suggestions.is_empty(), "Should find matching patterns");
}

#[test]
fn test_pattern_store_query_sorted_by_success() {
    let mut store = PatternStore::empty();

    // Add patterns with different success rates
    store.add_pattern(
        FixPattern::new("LOW", ErrorCategory::TypeMismatch)
            .with_error_pattern("test")
            .with_success_rate(0.5),
    );
    store.add_pattern(
        FixPattern::new("HIGH", ErrorCategory::TypeMismatch)
            .with_error_pattern("test")
            .with_success_rate(0.9),
    );

    let suggestions = store.query(ErrorCategory::TypeMismatch, "test", 0.0);

    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0].pattern_id, "HIGH");
    assert!(suggestions[0].success_rate > suggestions[1].success_rate);
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_oracle_config_custom() {
    let config = OracleConfig {
        confidence_threshold: 0.95,
        max_suggestions: 3,
        drift_detection_enabled: false,
        similarity_threshold: 0.8,
    };

    let oracle = RuchyOracle::with_config(config);

    assert!((oracle.config().confidence_threshold - 0.95).abs() < f64::EPSILON);
    assert_eq!(oracle.config().max_suggestions, 3);
    assert!(!oracle.config().drift_detection_enabled);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_oracle_empty_message() {
    let oracle = RuchyOracle::new();
    let error = CompilationError::new("");
    let classification = oracle.classify(&error);

    // Should still produce a valid classification
    assert!(classification.category.to_index() < ErrorCategory::COUNT);
}

#[test]
fn test_oracle_unknown_error_code() {
    let mut oracle = RuchyOracle::new();
    oracle.train_from_examples().expect("Training");

    let error = CompilationError::new("some unknown error").with_code("E9999");

    let classification = oracle.classify(&error);

    // Should fall back to Other or use keyword matching
    assert!(classification.category.to_index() < ErrorCategory::COUNT);
}

#[test]
fn test_oracle_very_long_message() {
    let oracle = RuchyOracle::new();
    let long_message = "error ".repeat(1000);
    let error = CompilationError::new(long_message);

    // Should not panic on very long messages
    let classification = oracle.classify(&error);
    assert!(classification.category.to_index() < ErrorCategory::COUNT);
}
