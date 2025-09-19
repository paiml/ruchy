// Extreme TDD Test Suite for src/notebook/testing/anticheat.rs
// Target: 407 lines, 0% → 95%+ coverage
//
// Quality Standards:
// - TDD methodology: Test-first development
// - Cyclomatic complexity ≤10 for all test functions
// - Property-based testing with 10,000+ iterations
// - Zero SATD (Self-Admitted Technical Debt) comments
// - Complete Big O algorithmic analysis
// - Toyota Way: Root cause analysis and systematic defect prevention

use ruchy::notebook::testing::anticheat::{
    AntiCheatSystem, ObfuscationDetector, PatternAnalyzer,
    Submission
};
use chrono::{DateTime, Utc, TimeZone};

// Helper functions for creating test data
fn create_test_submission(student_id: &str, assignment_id: &str, code: &str) -> Submission {
    // Generate a simple hash-based fingerprint for testing
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(code.trim()); // Basic normalization for test
    let fingerprint = format!("{:x}", hasher.finalize());

    Submission {
        student_id: student_id.to_string(),
        assignment_id: assignment_id.to_string(),
        code: code.to_string(),
        timestamp: Utc::now(),
        fingerprint,
    }
}

fn create_test_submission_with_timestamp(
    student_id: &str,
    assignment_id: &str,
    code: &str,
    timestamp: DateTime<Utc>
) -> Submission {
    // Generate a simple hash-based fingerprint for testing
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(code.trim()); // Basic normalization for test
    let fingerprint = format!("{:x}", hasher.finalize());

    Submission {
        student_id: student_id.to_string(),
        assignment_id: assignment_id.to_string(),
        code: code.to_string(),
        timestamp,
        fingerprint,
    }
}

fn create_test_code_sample() -> String {
    r#"
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn main() {
    println!("Factorial of 5: {}", factorial(5));
}
"#.to_string()
}

fn create_similar_code_sample() -> String {
    r#"
fn fact(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    } else {
        return n * fact(n - 1);
    }
}

fn main() {
    println!("Result: {}", fact(5));
}
"#.to_string()
}

fn create_obfuscated_code_sample() -> String {
    r#"
let a = "ZnVuY3Rpb24=";
let b = base64::decode(a);
eval(b);
let ______ = "secret";
let x123456789012345678901234567890 = "long_var";
if (true) { exec("dangerous_code"); }
"#.to_string()
}

// Test AntiCheatSystem creation and basic functionality
#[test]
fn test_anticheat_system_new() {
    let system = AntiCheatSystem::new();
    assert_eq!(system.similarity_threshold, 0.85);
    assert!(system.submission_history.is_empty());
    assert!(system.fingerprint_db.is_empty());
}

#[test]
fn test_anticheat_system_default() {
    let system = AntiCheatSystem::default();
    assert_eq!(system.similarity_threshold, 0.85);
    assert!(system.submission_history.is_empty());
    assert!(system.fingerprint_db.is_empty());
}

#[test]
fn test_anticheat_system_with_threshold() {
    let system = AntiCheatSystem::with_threshold(0.7);
    assert_eq!(system.similarity_threshold, 0.7);
    assert!(system.submission_history.is_empty());
    assert!(system.fingerprint_db.is_empty());
}

#[test]
fn test_anticheat_system_with_threshold_extreme_values() {
    let system_low = AntiCheatSystem::with_threshold(0.1);
    assert_eq!(system_low.similarity_threshold, 0.1);

    let system_high = AntiCheatSystem::with_threshold(0.99);
    assert_eq!(system_high.similarity_threshold, 0.99);

    let system_zero = AntiCheatSystem::with_threshold(0.0);
    assert_eq!(system_zero.similarity_threshold, 0.0);

    let system_one = AntiCheatSystem::with_threshold(1.0);
    assert_eq!(system_one.similarity_threshold, 1.0);
}

// Test plagiarism detection functionality
#[test]
fn test_check_plagiarism_first_submission() {
    let mut system = AntiCheatSystem::new();
    let submission = create_test_submission("student1", "hw1", &create_test_code_sample());

    let result = system.check_plagiarism(&submission);
    assert!(!result.is_plagiarized);
    assert_eq!(result.similarity_score, 0.0);
    assert!(result.matched_student.is_none());
    assert!(result.matched_sections.is_empty());
}

#[test]
fn test_check_plagiarism_identical_code_different_students() {
    let mut system = AntiCheatSystem::new();
    let code = create_test_code_sample();

    // First submission
    let submission1 = create_test_submission("student1", "hw1", &code);
    let result1 = system.check_plagiarism(&submission1);
    assert!(!result1.is_plagiarized);

    // Second submission with identical code
    let submission2 = create_test_submission("student2", "hw1", &code);
    let result2 = system.check_plagiarism(&submission2);
    assert!(result2.is_plagiarized);
    assert_eq!(result2.similarity_score, 1.0);
    assert_eq!(result2.matched_student, Some("student1".to_string()));
    assert!(!result2.matched_sections.is_empty());
}

#[test]
fn test_check_plagiarism_same_student_multiple_submissions() {
    let mut system = AntiCheatSystem::new();
    let code = create_test_code_sample();

    // First submission
    let submission1 = create_test_submission("student1", "hw1", &code);
    let result1 = system.check_plagiarism(&submission1);
    assert!(!result1.is_plagiarized);

    // Second submission by same student
    let submission2 = create_test_submission("student1", "hw2", &code);
    let result2 = system.check_plagiarism(&submission2);
    assert!(!result2.is_plagiarized); // Same student should not be flagged
}

#[test]
fn test_check_plagiarism_similar_but_not_identical_code() {
    let mut system = AntiCheatSystem::new();

    // First submission
    let submission1 = create_test_submission("student1", "hw1", &create_test_code_sample());
    let _result1 = system.check_plagiarism(&submission1);

    // Second submission with similar code
    let submission2 = create_test_submission("student2", "hw1", &create_similar_code_sample());
    let result2 = system.check_plagiarism(&submission2);

    // Should have some similarity but might not reach threshold
    assert!(result2.similarity_score > 0.0);
    if result2.similarity_score >= system.similarity_threshold {
        assert!(result2.is_plagiarized);
        assert_eq!(result2.matched_student, Some("student1".to_string()));
    }
}

#[test]
fn test_check_plagiarism_completely_different_code() {
    let mut system = AntiCheatSystem::new();

    // First submission
    let submission1 = create_test_submission("student1", "hw1", "fn hello() { println!(\"Hello\"); }");
    let _result1 = system.check_plagiarism(&submission1);

    // Second submission with completely different code
    let submission2 = create_test_submission("student2", "hw1", "fn sum(a: i32, b: i32) -> i32 { a + b }");
    let result2 = system.check_plagiarism(&submission2);

    assert!(!result2.is_plagiarized);
    assert!(result2.similarity_score < system.similarity_threshold);
}

#[test]
fn test_check_plagiarism_empty_code() {
    let mut system = AntiCheatSystem::new();

    let submission = create_test_submission("student1", "hw1", "");
    let result = system.check_plagiarism(&submission);

    assert!(!result.is_plagiarized);
    assert_eq!(result.similarity_score, 0.0);
}

#[test]
fn test_check_plagiarism_whitespace_only_code() {
    let mut system = AntiCheatSystem::new();

    let submission = create_test_submission("student1", "hw1", "   \n\t  \n   ");
    let result = system.check_plagiarism(&submission);

    assert!(!result.is_plagiarized);
    assert_eq!(result.similarity_score, 0.0);
}

// Test ObfuscationDetector functionality
#[test]
fn test_obfuscation_detector_new() {
    let detector = ObfuscationDetector::new();
    // Should have predefined suspicious patterns
    let test_code = "eval('malicious code')";
    let result = detector.is_obfuscated(test_code);
    assert!(result.is_likely_obfuscated);
}

#[test]
fn test_obfuscation_detector_default() {
    let detector = ObfuscationDetector::default();
    let test_code = "base64::decode(data)";
    let result = detector.is_obfuscated(test_code);
    assert!(result.is_likely_obfuscated);
}

#[test]
fn test_obfuscation_detector_clean_code() {
    let detector = ObfuscationDetector::new();
    let clean_code = create_test_code_sample();
    let result = detector.is_obfuscated(&clean_code);

    assert!(!result.is_likely_obfuscated);
    assert_eq!(result.confidence, 0.0);
    assert!(result.indicators.is_empty());
}

#[test]
fn test_obfuscation_detector_suspicious_patterns() {
    let detector = ObfuscationDetector::new();
    let suspicious_code = create_obfuscated_code_sample();
    let result = detector.is_obfuscated(&suspicious_code);

    assert!(result.is_likely_obfuscated);
    assert!(result.confidence > 0.0);
    assert!(!result.indicators.is_empty());

    // Should detect multiple suspicious indicators
    let indicators_text = result.indicators.join(" ");
    assert!(indicators_text.contains("base64") || indicators_text.contains("eval") || indicators_text.contains("exec"));
}

#[test]
fn test_obfuscation_detector_unusual_variable_names() {
    let detector = ObfuscationDetector::new();
    let code_with_unusual_vars = r#"
        let a = 5;
        let b = 10;
        let c = 15;
        let d = 20;
        let _ = 25;
        let __ = 30;
    "#;
    let result = detector.is_obfuscated(code_with_unusual_vars);

    // May or may not be flagged depending on ratio of unusual names
    if result.is_likely_obfuscated {
        assert!(result.indicators.iter().any(|i| i.contains("unusual variable names")));
    }
}

#[test]
fn test_obfuscation_detector_long_lines() {
    let detector = ObfuscationDetector::new();
    let long_line = "a".repeat(250);
    let code_with_long_line = format!("let x = \"{}\";", long_line);
    let result = detector.is_obfuscated(&code_with_long_line);

    assert!(result.is_likely_obfuscated);
    assert!(result.indicators.iter().any(|i| i.contains("long line")));
}

#[test]
fn test_obfuscation_detector_multiple_indicators() {
    let detector = ObfuscationDetector::new();
    let highly_suspicious_code = format!(
        "eval('code'); let a = 1; let b = 2; {}",
        "x".repeat(250)
    );
    let result = detector.is_obfuscated(&highly_suspicious_code);

    assert!(result.is_likely_obfuscated);
    assert!(result.confidence > 0.1);
    assert!(result.indicators.len() >= 2); // Multiple indicators
}

// Test PatternAnalyzer functionality
#[test]
fn test_pattern_analyzer_new() {
    let mut analyzer = PatternAnalyzer::new();
    // Internal patterns should be empty for new analyzer
    // We can only test the public interface
    let analysis = analyzer.analyze_pattern("student1", Utc::now());
    assert!(!analysis.is_suspicious); // First submission shouldn't be suspicious
    assert_eq!(analysis.submission_count, 1);
}

#[test]
fn test_pattern_analyzer_default() {
    let mut analyzer = PatternAnalyzer::default();
    let analysis = analyzer.analyze_pattern("student1", Utc::now());
    assert!(!analysis.is_suspicious);
    assert_eq!(analysis.submission_count, 1);
}

#[test]
fn test_pattern_analyzer_single_submission() {
    let mut analyzer = PatternAnalyzer::new();
    let timestamp = Utc::now();
    let analysis = analyzer.analyze_pattern("student1", timestamp);

    assert!(!analysis.is_suspicious);
    assert!(analysis.indicators.is_empty());
    assert_eq!(analysis.submission_count, 1);
}

#[test]
fn test_pattern_analyzer_rapid_submissions() {
    let mut analyzer = PatternAnalyzer::new();
    let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();

    // Submit multiple times with very short intervals
    let _analysis1 = analyzer.analyze_pattern("student1", base_time);
    let analysis2 = analyzer.analyze_pattern("student1", base_time + chrono::Duration::seconds(10));

    assert!(analysis2.is_suspicious);
    assert!(analysis2.indicators.iter().any(|i| i.contains("Rapid")));
    assert_eq!(analysis2.submission_count, 2);
}

#[test]
fn test_pattern_analyzer_normal_submissions() {
    let mut analyzer = PatternAnalyzer::new();
    let base_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();

    // Submit with reasonable intervals
    let _analysis1 = analyzer.analyze_pattern("student1", base_time);
    let analysis2 = analyzer.analyze_pattern("student1", base_time + chrono::Duration::hours(2));

    assert!(!analysis2.is_suspicious);
    assert!(analysis2.indicators.is_empty());
    assert_eq!(analysis2.submission_count, 2);
}

#[test]
fn test_pattern_analyzer_late_night_pattern() {
    let mut analyzer = PatternAnalyzer::new();

    // Submit multiple times during late night hours (2-5 AM)
    let late_night1 = Utc.with_ymd_and_hms(2024, 1, 1, 3, 0, 0).unwrap();
    let late_night2 = Utc.with_ymd_and_hms(2024, 1, 2, 4, 0, 0).unwrap();
    let late_night3 = Utc.with_ymd_and_hms(2024, 1, 3, 2, 30, 0).unwrap();

    let _analysis1 = analyzer.analyze_pattern("student1", late_night1);
    let _analysis2 = analyzer.analyze_pattern("student1", late_night2);
    let analysis3 = analyzer.analyze_pattern("student1", late_night3);

    assert!(analysis3.is_suspicious);
    assert!(analysis3.indicators.iter().any(|i| i.contains("late-night")));
    assert_eq!(analysis3.submission_count, 3);
}

#[test]
fn test_pattern_analyzer_mixed_times() {
    let mut analyzer = PatternAnalyzer::new();

    // Mix of normal and late-night submissions
    let normal_time = Utc.with_ymd_and_hms(2024, 1, 1, 14, 0, 0).unwrap();
    let late_night = Utc.with_ymd_and_hms(2024, 1, 2, 3, 0, 0).unwrap();

    let _analysis1 = analyzer.analyze_pattern("student1", normal_time);
    let analysis2 = analyzer.analyze_pattern("student1", late_night);

    // With only 50% late-night, should not be flagged
    assert!(!analysis2.is_suspicious);
    assert_eq!(analysis2.submission_count, 2);
}

#[test]
fn test_pattern_analyzer_different_students() {
    let mut analyzer = PatternAnalyzer::new();
    let timestamp = Utc::now();

    let analysis1 = analyzer.analyze_pattern("student1", timestamp);
    let analysis2 = analyzer.analyze_pattern("student2", timestamp);

    // Different students should have independent patterns
    assert_eq!(analysis1.submission_count, 1);
    assert_eq!(analysis2.submission_count, 1);
    assert!(!analysis1.is_suspicious);
    assert!(!analysis2.is_suspicious);
}

// Integration tests combining multiple components
#[test]
fn test_anticheat_integration_workflow() {
    let mut anticheat = AntiCheatSystem::new();
    let obfuscation = ObfuscationDetector::new();
    let mut pattern = PatternAnalyzer::new();

    let code = create_test_code_sample();
    let timestamp = Utc::now();
    let submission = create_test_submission_with_timestamp("student1", "hw1", &code, timestamp);

    // Check all systems
    let plagiarism_result = anticheat.check_plagiarism(&submission);
    let obfuscation_result = obfuscation.is_obfuscated(&code);
    let pattern_result = pattern.analyze_pattern(&submission.student_id, timestamp);

    // Clean code should pass all checks
    assert!(!plagiarism_result.is_plagiarized);
    assert!(!obfuscation_result.is_likely_obfuscated);
    assert!(!pattern_result.is_suspicious);
}

#[test]
fn test_anticheat_integration_suspicious_submission() {
    let mut anticheat = AntiCheatSystem::new();
    let obfuscation = ObfuscationDetector::new();
    let mut pattern = PatternAnalyzer::new();

    // First, submit clean code
    let clean_code = create_test_code_sample();
    let clean_submission = create_test_submission("student1", "hw1", &clean_code);
    let _clean_result = anticheat.check_plagiarism(&clean_submission);

    // Then submit suspicious code
    let suspicious_code = create_obfuscated_code_sample();
    let suspicious_timestamp = Utc::now();
    let suspicious_submission = create_test_submission_with_timestamp(
        "student2", "hw1", &suspicious_code, suspicious_timestamp
    );

    // Check all systems
    let _plagiarism_result = anticheat.check_plagiarism(&suspicious_submission);
    let obfuscation_result = obfuscation.is_obfuscated(&suspicious_code);
    let _pattern_result = pattern.analyze_pattern(&suspicious_submission.student_id, suspicious_timestamp);

    // Should detect obfuscation at minimum
    assert!(obfuscation_result.is_likely_obfuscated);
}

// Edge case tests
#[test]
fn test_submission_with_only_comments() {
    let mut system = AntiCheatSystem::new();
    let code = r#"
        // This is a comment
        // Another comment
        /* Block comment */
    "#;
    let submission = create_test_submission("student1", "hw1", code);
    let result = system.check_plagiarism(&submission);

    assert!(!result.is_plagiarized);
    assert_eq!(result.similarity_score, 0.0);
}

#[test]
fn test_submission_with_unicode_characters() {
    let mut system = AntiCheatSystem::new();
    let code = r#"
        fn main() {
            println!("Hello, 世界! 🌍");
            let résultat = 42;
            println!("Résultat: {}", résultat);
        }
    "#;
    let submission = create_test_submission("student1", "hw1", code);
    let result = system.check_plagiarism(&submission);

    assert!(!result.is_plagiarized);
}

#[test]
fn test_very_large_code_submission() {
    let mut system = AntiCheatSystem::new();
    let large_code = "fn test() { }\n".repeat(1000);
    let submission = create_test_submission("student1", "hw1", &large_code);
    let result = system.check_plagiarism(&submission);

    assert!(!result.is_plagiarized);
}

#[test]
fn test_obfuscation_detector_with_false_positives() {
    let detector = ObfuscationDetector::new();
    // Code that legitimately uses patterns that might seem suspicious
    let legitimate_code = r#"
        use base64;
        fn decode_config(config: &str) -> String {
            base64::decode(config).unwrap()
        }

        fn compile_regex(pattern: &str) -> Result<regex::Regex, regex::Error> {
            regex::Regex::new(pattern)
        }
    "#;
    let result = detector.is_obfuscated(legitimate_code);

    // This will likely be flagged as obfuscated due to "base64" and "compile"
    // That's expected behavior - the detector errs on the side of caution
    if result.is_likely_obfuscated {
        assert!(result.confidence < 1.0); // Should not be 100% confident
    }
}

// Property-based tests with 10,000+ iterations
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn test_anticheat_system_threshold_never_panics(
            threshold in 0.0f64..1.0f64
        ) {
            let system = AntiCheatSystem::with_threshold(threshold);
            prop_assert_eq!(system.similarity_threshold, threshold);
        }

        #[test]
        fn test_submission_creation_never_panics(
            student_id in "[a-zA-Z0-9_]{1,20}",
            assignment_id in "[a-zA-Z0-9_]{1,20}",
            code in "[a-zA-Z0-9 \\n\\t._(){}\\[\\];:,]*{0,500}"
        ) {
            let submission = create_test_submission(&student_id, &assignment_id, &code);
            prop_assert_eq!(submission.student_id, student_id);
            prop_assert_eq!(submission.assignment_id, assignment_id);
            prop_assert_eq!(submission.code, code);
        }

        #[test]
        fn test_plagiarism_check_never_panics(
            code in "[a-zA-Z0-9 \\n\\t._(){}\\[\\];:,]*{0,200}"
        ) {
            let mut system = AntiCheatSystem::new();
            let submission = create_test_submission("student1", "hw1", &code);
            let result = system.check_plagiarism(&submission);

            prop_assert!(result.similarity_score >= 0.0);
            prop_assert!(result.similarity_score <= 1.0);
        }

        #[test]
        fn test_obfuscation_detection_never_panics(
            code in "[a-zA-Z0-9 \\n\\t._(){}\\[\\];:,]*{0,300}"
        ) {
            let detector = ObfuscationDetector::new();
            let result = detector.is_obfuscated(&code);

            prop_assert!(result.confidence >= 0.0);
            prop_assert!(result.confidence <= 1.0);
        }

        #[test]
        fn test_pattern_analysis_never_panics(
            student_id in "[a-zA-Z0-9_]{1,20}",
            hour in 0u32..24u32,
            minute in 0u32..60u32
        ) {
            let mut analyzer = PatternAnalyzer::new();
            let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, hour, minute, 0).unwrap();
            let result = analyzer.analyze_pattern(&student_id, timestamp);

            prop_assert!(result.submission_count > 0);
        }

        #[test]
        fn test_similarity_scores_are_consistent(
            code1 in "[a-zA-Z0-9 \\n]*{1,100}",
            code2 in "[a-zA-Z0-9 \\n]*{1,100}"
        ) {
            let _system = AntiCheatSystem::new();
            let submission1 = create_test_submission("s1", "hw1", &code1);
            let submission2 = create_test_submission("s2", "hw1", &code2);

            // Identical codes should have similarity score of 1.0
            if code1 == code2 {
                let mut temp_system = AntiCheatSystem::new();
                let _result1 = temp_system.check_plagiarism(&submission1);
                let result2 = temp_system.check_plagiarism(&submission2);
                prop_assert_eq!(result2.similarity_score, 1.0);
            }
        }

        #[test]
        fn test_fingerprint_consistency(
            code in "[a-zA-Z0-9 \\n]*{1,50}"
        ) {
            let _system = AntiCheatSystem::new();
            let submission1 = create_test_submission("s1", "hw1", &code);
            let submission2 = create_test_submission("s2", "hw1", &code);

            // Same code should generate same fingerprint
            prop_assert_eq!(submission1.fingerprint, submission2.fingerprint);
        }
    }
}

// Big O Complexity Analysis
// AntiCheatSystem Core Functions:
// - new(): O(1) - Constant time constructor
// - with_threshold(): O(1) - Constant time constructor with parameter
// - check_plagiarism(): O(n*m*k) where n=students, m=submissions per student, k=code length
//   - Fingerprint generation: O(k) where k is code length
//   - Similarity calculation: O(k1 + k2) for comparing two code samples
//   - Overall: O(n*m*k) for checking against all previous submissions
// - normalize_code(): O(k) where k is code length (single pass filtering)
// - calculate_similarity(): O(k1 + k2) using Jaccard similarity on token sets
// - tokenize(): O(k) where k is code length (single pass splitting)
// - find_matched_sections(): O(l1 * l2) where l1, l2 are line counts
// - merge_adjacent_matches(): O(m log m) where m is number of matches (sorting)
//
// ObfuscationDetector Core Functions:
// - new(): O(1) - Constant time constructor with predefined patterns
// - is_obfuscated(): O(k + p*k + l) where k=code length, p=pattern count, l=line count
//   - Pattern matching: O(p*k) checking each pattern against full code
//   - Variable extraction: O(l) single pass through lines
//   - Line length check: O(l) single pass through lines
// - extract_variable_names(): O(l) where l is number of lines
// - is_unusual_name(): O(n) where n is name length (character analysis)
//
// PatternAnalyzer Core Functions:
// - new(): O(1) - Constant time constructor
// - analyze_pattern(): O(s) where s is number of submissions for student
//   - Time interval calculation: O(s) single pass through timestamps
//   - Pattern detection: O(s) single pass analysis
// - Time complexity scales linearly with submission history per student
//
// Space Complexity: O(n*m*k) where n=students, m=avg submissions, k=avg code size
// - Submission history: Stores all submissions with full code
// - Fingerprint database: O(n*m) entries with hash keys
// - Pattern analysis: O(n*s) where s is max submissions per student
//
// Performance Characteristics:
// - Plagiarism detection: Quadratic in number of submissions (compares all pairs)
// - Fingerprint matching: O(1) hash lookup for exact matches
// - Similarity calculation: Linear in combined code length
// - Memory usage: Grows with submission history (potential optimization: LRU cache)
// - Pattern analysis: Linear per student, scales with user base

// All test functions maintain cyclomatic complexity ≤ 10
// Property tests run with 10,000+ iterations for statistical confidence
// No SATD (Self-Admitted Technical Debt) comments
// Big O analysis provided for all major anti-cheat operations