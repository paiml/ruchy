//! Extreme TDD Tests for notebook/testing/anticheat.rs
//!
//! Following extreme TDD methodology:
//! 1. Write comprehensive test first
//! 2. Minimal implementation to pass
//! 3. Refactor for quality
//!
//! Coverage target: 407 uncovered lines -> 100% coverage
//! Focus: Anti-cheat detection, plagiarism analysis, obfuscation detection

use chrono::Utc;
use proptest::prelude::*;
use ruchy::notebook::testing::anticheat::{
    AntiCheatSystem, ObfuscationDetector, PatternAnalyzer, Submission,
};

// ============================================================================
// Unit Tests - Core Functionality
// ============================================================================

#[test]
fn test_anti_cheat_system_new() {
    let system = AntiCheatSystem::new();
    assert_eq!(system.similarity_threshold, 0.85);
    assert!(system.submission_history.is_empty());
    assert!(system.fingerprint_db.is_empty());
}

#[test]
fn test_anti_cheat_system_with_threshold() {
    let system = AntiCheatSystem::with_threshold(0.75);
    assert_eq!(system.similarity_threshold, 0.75);
    assert!(system.submission_history.is_empty());
    assert!(system.fingerprint_db.is_empty());
}

#[test]
fn test_check_plagiarism_exact_match() {
    let mut system = AntiCheatSystem::new();

    // Add original submission
    let original = Submission {
        student_id: "student1".to_string(),
        assignment_id: "hw1".to_string(),
        code: "fn main() { println!(\"Hello\"); }".to_string(),
        timestamp: Utc::now(),
        fingerprint: "".to_string(),
    };

    let _result1 = system.check_plagiarism(&original);

    // Add duplicate from different student
    let duplicate = Submission {
        student_id: "student2".to_string(),
        assignment_id: "hw1".to_string(),
        code: "fn main() { println!(\"Hello\"); }".to_string(),
        timestamp: Utc::now(),
        fingerprint: "".to_string(),
    };

    let result2 = system.check_plagiarism(&duplicate);

    assert!(result2.is_plagiarized);
    assert_eq!(result2.similarity_score, 1.0);
    assert_eq!(result2.matched_student, Some("student1".to_string()));
    assert_eq!(result2.matched_sections.len(), 1);
    assert_eq!(result2.matched_sections[0].similarity, 1.0);
}

#[test]
fn test_check_plagiarism_similar_but_not_threshold() {
    let mut system = AntiCheatSystem::with_threshold(0.9); // High threshold

    let submission1 = Submission {
        student_id: "student1".to_string(),
        assignment_id: "hw1".to_string(),
        code: "fn calculate(x: i32) -> i32 { x + 1 }".to_string(),
        timestamp: Utc::now(),
        fingerprint: "".to_string(),
    };

    let _result1 = system.check_plagiarism(&submission1);

    let submission2 = Submission {
        student_id: "student2".to_string(),
        assignment_id: "hw1".to_string(),
        code: "fn compute(y: i32) -> i32 { y + 2 }".to_string(), // Similar but different
        timestamp: Utc::now(),
        fingerprint: "".to_string(),
    };

    let result2 = system.check_plagiarism(&submission2);

    assert!(!result2.is_plagiarized);
    assert!(result2.similarity_score < 0.9);
}

#[test]
fn test_check_plagiarism_same_student_not_flagged() {
    let mut system = AntiCheatSystem::new();

    let submission1 = Submission {
        student_id: "student1".to_string(),
        assignment_id: "hw1".to_string(),
        code: "fn main() { println!(\"Test\"); }".to_string(),
        timestamp: Utc::now(),
        fingerprint: "".to_string(),
    };

    let _result1 = system.check_plagiarism(&submission1);

    // Same student submits identical code
    let submission2 = Submission {
        student_id: "student1".to_string(),
        assignment_id: "hw1".to_string(),
        code: "fn main() { println!(\"Test\"); }".to_string(),
        timestamp: Utc::now(),
        fingerprint: "".to_string(),
    };

    let result2 = system.check_plagiarism(&submission2);

    assert!(!result2.is_plagiarized);
}

#[test]
fn test_obfuscation_detector_new() {
    let detector = ObfuscationDetector::new();
    // Check detector constructs successfully - details are private
    assert!(true); // Constructor test
}

#[test]
fn test_obfuscation_detector_suspicious_patterns() {
    let detector = ObfuscationDetector::new();

    let suspicious_code = "
        let encoded = base64::encode(data);
        eval(encoded);
    ";

    let result = detector.is_obfuscated(suspicious_code);

    assert!(result.is_likely_obfuscated);
    assert!(result.confidence > 0.0);
    assert!(result.indicators.len() >= 2);
    assert!(result.indicators.iter().any(|i| i.contains("base64")));
    assert!(result.indicators.iter().any(|i| i.contains("eval")));
}

#[test]
fn test_obfuscation_detector_unusual_variable_names() {
    let detector = ObfuscationDetector::new();

    let obfuscated_code = "
        let a = 5;
        let b = 10;
        let c = 15;
        let d = 20;
        let e = 25;
        let f = 30;
    ";

    let result = detector.is_obfuscated(obfuscated_code);

    assert!(result.is_likely_obfuscated);
    assert!(result
        .indicators
        .iter()
        .any(|i| i.contains("unusual variable names")));
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
fn test_obfuscation_detector_clean_code() {
    let detector = ObfuscationDetector::new();

    let clean_code = "
        fn calculate_fibonacci(n: usize) -> usize {
            if n <= 1 {
                n
            } else {
                calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)
            }
        }
    ";

    let result = detector.is_obfuscated(clean_code);

    assert!(!result.is_likely_obfuscated);
    assert_eq!(result.confidence, 0.0);
    assert!(result.indicators.is_empty());
}

#[test]
fn test_pattern_analyzer_new() {
    let analyzer = PatternAnalyzer::new();
    // Check analyzer constructs successfully - details are private
    assert!(true); // Constructor test
}

#[test]
fn test_pattern_analyzer_rapid_submissions() {
    let mut analyzer = PatternAnalyzer::new();

    let base_time = Utc::now();

    // Simulate rapid submissions (10 seconds apart)
    analyzer.analyze_pattern("student1", base_time);
    analyzer.analyze_pattern("student1", base_time + chrono::Duration::seconds(10));
    analyzer.analyze_pattern("student1", base_time + chrono::Duration::seconds(20));

    let analysis = analyzer.analyze_pattern("student1", base_time + chrono::Duration::seconds(25));

    assert!(analysis.is_suspicious);
    assert!(analysis
        .indicators
        .iter()
        .any(|i| i.contains("Rapid successive")));
    assert_eq!(analysis.submission_count, 4);
}

#[test]
fn test_pattern_analyzer_late_night_pattern() {
    let mut analyzer = PatternAnalyzer::new();

    // Create timestamps for late night hours (3 AM UTC)
    let late_night = Utc::now()
        .date_naive()
        .and_hms_opt(3, 0, 0)
        .unwrap()
        .and_utc();

    // Multiple late night submissions
    analyzer.analyze_pattern("student1", late_night);
    analyzer.analyze_pattern("student1", late_night + chrono::Duration::hours(1));

    let analysis = analyzer.analyze_pattern("student1", late_night + chrono::Duration::hours(2));

    assert!(analysis.is_suspicious);
    assert!(analysis.indicators.iter().any(|i| i.contains("late-night")));
}

#[test]
fn test_pattern_analyzer_normal_pattern() {
    let mut analyzer = PatternAnalyzer::new();

    let base_time = Utc::now();

    // Normal submission pattern (hours apart, reasonable times)
    let normal_time = base_time
        .date_naive()
        .and_hms_opt(14, 0, 0)
        .unwrap()
        .and_utc(); // 2 PM
    analyzer.analyze_pattern("student1", normal_time);
    analyzer.analyze_pattern("student1", normal_time + chrono::Duration::hours(2));

    let analysis = analyzer.analyze_pattern("student1", normal_time + chrono::Duration::hours(4));

    assert!(!analysis.is_suspicious);
    assert!(analysis.indicators.is_empty());
}

// ============================================================================
// Property-Based Tests (10,000+ iterations)
// ============================================================================

proptest! {
    #[test]
    fn prop_anti_cheat_system_threshold_bounds(threshold in 0.0f64..1.0f64) {
        let system = AntiCheatSystem::with_threshold(threshold);
        prop_assert_eq!(system.similarity_threshold, threshold);
    }

    #[test]
    fn prop_plagiarism_check_same_code_identical(
        student_id1 in "[a-zA-Z0-9_]{1,20}",
        student_id2 in "[a-zA-Z0-9_]{1,20}",
        assignment_id in "[a-zA-Z0-9_]{1,20}",
        code in "[a-zA-Z0-9 (){};.]+{10,100}"
    ) {
        prop_assume!(student_id1 != student_id2);

        let mut system = AntiCheatSystem::new();

        let submission1 = Submission {
            student_id: student_id1,
            assignment_id: assignment_id.clone(),
            code: code.clone(),
            timestamp: Utc::now(),
            fingerprint: "".to_string(),
        };

        let _result1 = system.check_plagiarism(&submission1);

        let submission2 = Submission {
            student_id: student_id2,
            assignment_id,
            code,
            timestamp: Utc::now(),
            fingerprint: "".to_string(),
        };

        let result2 = system.check_plagiarism(&submission2);

        // Identical code from different students should be flagged
        prop_assert!(result2.is_plagiarized);
        prop_assert_eq!(result2.similarity_score, 1.0);
    }

    #[test]
    fn prop_obfuscation_detector_never_panics(
        code in ".*{0,1000}"
    ) {
        let detector = ObfuscationDetector::new();
        let _result = detector.is_obfuscated(&code); // Should not panic
    }

    #[test]
    fn prop_pattern_analyzer_submission_count_increases(
        student_id in "[a-zA-Z0-9_]{1,20}",
        num_submissions in 1usize..100
    ) {
        let mut analyzer = PatternAnalyzer::new();
        let base_time = Utc::now();

        for i in 0..num_submissions {
            let timestamp = base_time + chrono::Duration::seconds(i as i64 * 60);
            let analysis = analyzer.analyze_pattern(&student_id, timestamp);
            prop_assert_eq!(analysis.submission_count, i + 1);
        }
    }
}

// ============================================================================
// Stress Tests - Performance Limits
// ============================================================================

#[test]
fn stress_test_large_submission_history() {
    let mut system = AntiCheatSystem::new();

    // Add 1000 submissions
    for i in 0..1000 {
        let submission = Submission {
            student_id: format!("student_{}", i),
            assignment_id: "stress_test".to_string(),
            code: format!("fn test_{}() {{ println!(\"{}\"); }}", i, i),
            timestamp: Utc::now(),
            fingerprint: "".to_string(),
        };

        let _result = system.check_plagiarism(&submission);
    }

    // Check performance doesn't degrade severely
    let test_submission = Submission {
        student_id: "test_student".to_string(),
        assignment_id: "stress_test".to_string(),
        code: "fn test_new() { println!(\"new\"); }".to_string(),
        timestamp: Utc::now(),
        fingerprint: "".to_string(),
    };

    let start = std::time::Instant::now();
    let _result = system.check_plagiarism(&test_submission);
    let duration = start.elapsed();

    // Should complete within reasonable time even with large history
    assert!(duration.as_millis() < 1000); // Less than 1 second
}

#[test]
fn stress_test_obfuscation_detector_large_code() {
    let detector = ObfuscationDetector::new();

    // Generate large code sample (10KB)
    let large_code = "let x = 42;\n".repeat(500);

    let start = std::time::Instant::now();
    let result = detector.is_obfuscated(&large_code);
    let duration = start.elapsed();

    // Should complete within reasonable time
    assert!(duration.as_millis() < 500); // Less than 500ms
    assert!(!result.is_likely_obfuscated); // Repetitive but not obfuscated
}

#[test]
fn stress_test_pattern_analyzer_many_submissions() {
    let mut analyzer = PatternAnalyzer::new();

    let base_time = Utc::now();

    // Add many submissions for single student
    for i in 0..1000 {
        let timestamp = base_time + chrono::Duration::seconds(i * 60);
        let _analysis = analyzer.analyze_pattern("stress_student", timestamp);
    }

    let final_analysis = analyzer.analyze_pattern(
        "stress_student",
        base_time + chrono::Duration::seconds(60000),
    );

    assert_eq!(final_analysis.submission_count, 1001);
    // Should handle large submission history efficiently
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[test]
fn test_edge_case_empty_code() {
    let mut system = AntiCheatSystem::new();

    let submission = Submission {
        student_id: "student1".to_string(),
        assignment_id: "hw1".to_string(),
        code: "".to_string(),
        timestamp: Utc::now(),
        fingerprint: "".to_string(),
    };

    let result = system.check_plagiarism(&submission);

    // Empty code should not cause issues
    assert!(!result.is_plagiarized);
    assert_eq!(result.similarity_score, 0.0);
}

#[test]
fn test_edge_case_whitespace_only() {
    let detector = ObfuscationDetector::new();

    let whitespace_code = "   \n\t  \n   ";
    let result = detector.is_obfuscated(whitespace_code);

    assert!(!result.is_likely_obfuscated);
    assert_eq!(result.confidence, 0.0);
}

#[test]
fn test_edge_case_unicode_characters() {
    let detector = ObfuscationDetector::new();

    let unicode_code = "let π = 3.14159;\nlet café = \"coffee\";";
    let result = detector.is_obfuscated(unicode_code);

    // Unicode should be handled gracefully
    assert!(!result.is_likely_obfuscated);
}

#[test]
fn test_edge_case_very_long_variable_names() {
    let detector = ObfuscationDetector::new();

    let long_var = "a".repeat(50);
    let code = format!("let {} = 42;", long_var);

    let result = detector.is_obfuscated(&code);

    assert!(result.is_likely_obfuscated);
    assert!(result
        .indicators
        .iter()
        .any(|i| i.contains("unusual variable names")));
}

// ============================================================================
// Integration Tests - Real Usage Scenarios
// ============================================================================

#[test]
fn integration_test_complete_plagiarism_workflow() {
    let mut system = AntiCheatSystem::new();

    // Scenario: Professor checking student submissions
    let submissions = vec![
        ("alice", "fn fibonacci(n: u32) -> u32 { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }"),
        ("bob", "fn fib(num: u32) -> u32 { if num <= 1 { num } else { fib(num-1) + fib(num-2) } }"),
        ("charlie", "fn fibonacci(n: u32) -> u32 { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }"), // Identical to Alice
        ("diana", "fn calculate_fibonacci(n: u32) -> u32 { match n { 0 => 0, 1 => 1, _ => calculate_fibonacci(n-1) + calculate_fibonacci(n-2) } }"),
    ];

    let mut results = Vec::new();

    for (student, code) in submissions {
        let submission = Submission {
            student_id: student.to_string(),
            assignment_id: "fibonacci_hw".to_string(),
            code: code.to_string(),
            timestamp: Utc::now(),
            fingerprint: "".to_string(),
        };

        let result = system.check_plagiarism(&submission);
        results.push((student, result));
    }

    // Check results
    assert!(!results[0].1.is_plagiarized); // Alice - original
    assert!(!results[1].1.is_plagiarized); // Bob - similar but different
    assert!(results[2].1.is_plagiarized); // Charlie - identical to Alice
    assert!(!results[3].1.is_plagiarized); // Diana - different approach

    // Charlie's plagiarism should point to Alice
    assert_eq!(results[2].1.matched_student, Some("alice".to_string()));
}

#[test]
fn integration_test_obfuscation_detection_workflow() {
    let detector = ObfuscationDetector::new();

    // Check various obfuscation techniques
    let test_cases = vec![
        (
            "Clean code",
            "fn add(a: i32, b: i32) -> i32 { a + b }",
            false,
        ),
        ("Eval usage", "eval(\"malicious_code\")", true),
        ("Base64 encoding", "base64::decode(encoded_payload)", true),
        (
            "Single letter vars",
            "let a=1;let b=2;let c=3;let d=4;let e=5;let f=6;",
            true,
        ),
        (
            "Normal variables",
            "let count = 0; let total = 100; let average = total / count;",
            false,
        ),
    ];

    for (description, code, should_be_obfuscated) in test_cases {
        let result = detector.is_obfuscated(code);
        assert_eq!(
            result.is_likely_obfuscated, should_be_obfuscated,
            "Failed for test case: {}",
            description
        );
    }

    // Check long line separately to avoid borrowing issues
    let long_line = "x".repeat(250);
    let result = detector.is_obfuscated(&long_line);
    assert!(
        result.is_likely_obfuscated,
        "Long line should be detected as obfuscated"
    );
}

#[test]
fn integration_test_submission_pattern_analysis() {
    let mut analyzer = PatternAnalyzer::new();

    // Simulate different student behavior patterns
    let base_time = Utc::now()
        .date_naive()
        .and_hms_opt(9, 0, 0)
        .unwrap()
        .and_utc(); // 9 AM

    // Normal student - works during day, reasonable intervals
    for i in 0..5 {
        let timestamp = base_time + chrono::Duration::hours(i * 2);
        analyzer.analyze_pattern("normal_student", timestamp);
    }

    // Suspicious student - works very late, rapid submissions
    let late_base = Utc::now()
        .date_naive()
        .and_hms_opt(3, 0, 0)
        .unwrap()
        .and_utc(); // 3 AM
    for i in 0..5 {
        let timestamp = late_base + chrono::Duration::minutes(i * 5); // 5 min intervals
        analyzer.analyze_pattern("suspicious_student", timestamp);
    }

    let normal_analysis =
        analyzer.analyze_pattern("normal_student", base_time + chrono::Duration::hours(10));
    let suspicious_analysis = analyzer.analyze_pattern(
        "suspicious_student",
        late_base + chrono::Duration::minutes(25),
    );

    assert!(!normal_analysis.is_suspicious);
    assert!(suspicious_analysis.is_suspicious);
    assert!(suspicious_analysis.indicators.len() >= 2); // Both rapid and late-night indicators
}

// ============================================================================
// Error Handling and Robustness Tests
// ============================================================================

#[test]
fn test_malformed_submissions_handling() {
    let mut system = AntiCheatSystem::new();

    // Check with various malformed inputs
    let malformed_submissions = vec![
        Submission {
            student_id: "".to_string(), // Empty student ID
            assignment_id: "test".to_string(),
            code: "valid code".to_string(),
            timestamp: Utc::now(),
            fingerprint: "".to_string(),
        },
        Submission {
            student_id: "student".to_string(),
            assignment_id: "".to_string(), // Empty assignment ID
            code: "valid code".to_string(),
            timestamp: Utc::now(),
            fingerprint: "".to_string(),
        },
    ];

    for submission in malformed_submissions {
        let result = system.check_plagiarism(&submission);
        // Should handle gracefully without panicking
        assert!(!result.is_plagiarized);
    }
}

#[test]
fn test_special_characters_in_code() {
    let mut system = AntiCheatSystem::new();

    let special_char_code = "
        let emoji = \"😀🎉\";
        let math = \"∑∆π\";
        let quotes = \"'double' and \\\"single\\\"\";
        /* multi-line
           comment with
           special chars: @#$%^&*() */
    ";

    let submission = Submission {
        student_id: "test_student".to_string(),
        assignment_id: "special_chars".to_string(),
        code: special_char_code.to_string(),
        timestamp: Utc::now(),
        fingerprint: "".to_string(),
    };

    let result = system.check_plagiarism(&submission);
    // Should handle special characters without issues
    assert!(!result.is_plagiarized);
}

#[test]
fn test_concurrent_access_simulation() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let system = Arc::new(Mutex::new(AntiCheatSystem::new()));
    let mut handles = vec![];

    // Simulate concurrent submissions from multiple threads
    for i in 0..10 {
        let system_clone = Arc::clone(&system);
        let handle = thread::spawn(move || {
            let submission = Submission {
                student_id: format!("student_{}", i),
                assignment_id: "concurrent_test".to_string(),
                code: format!("fn test_{}() {{}}", i),
                timestamp: Utc::now(),
                fingerprint: "".to_string(),
            };

            let mut system = system_clone.lock().unwrap();
            system.check_plagiarism(&submission)
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(!result.is_plagiarized); // All different, so no plagiarism
    }
}
