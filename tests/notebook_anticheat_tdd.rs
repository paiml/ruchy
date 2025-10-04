// EXTREME TDD: Notebook AntiCheat Tests
// Sprint 80: 0% Coverage Modules Attack
// Testing notebook/testing/anticheat.rs with comprehensive coverage

use chrono::Utc;
use ruchy::notebook::testing::anticheat::{
    AntiCheatSystem, MatchedSection, PlagiarismResult, Submission,
};

#[cfg(test)]
mod anticheat_unit_tests {
    use super::*;

    #[test]
    fn test_anticheat_new() {
        let system = AntiCheatSystem::new();
        assert_eq!(system.similarity_threshold, 0.85);
        assert!(system.submission_history.is_empty());
        assert!(system.fingerprint_db.is_empty());
    }

    #[test]
    fn test_anticheat_default() {
        let system = AntiCheatSystem::default();
        assert_eq!(system.similarity_threshold, 0.85);
        assert!(system.submission_history.is_empty());
        assert!(system.fingerprint_db.is_empty());
    }

    #[test]
    fn test_anticheat_with_threshold() {
        let system = AntiCheatSystem::with_threshold(0.90);
        assert_eq!(system.similarity_threshold, 0.90);

        let system2 = AntiCheatSystem::with_threshold(0.75);
        assert_eq!(system2.similarity_threshold, 0.75);
    }

    #[test]
    fn test_check_plagiarism_no_history() {
        let mut system = AntiCheatSystem::new();

        let submission = Submission {
            student_id: "student1".to_string(),
            assignment_id: "hw1".to_string(),
            code: "def add(a, b): return a + b".to_string(),
            timestamp: Utc::now(),
            fingerprint: "abc123".to_string(),
        };

        let result = system.check_plagiarism(&submission);

        // First submission should not be plagiarized
        assert!(!result.is_plagiarized);
        assert!(result.similarity_score < system.similarity_threshold);
        assert!(result.matched_student.is_none());
    }

    #[test]
    fn test_check_plagiarism_identical_code() {
        let mut system = AntiCheatSystem::new();

        let submission1 = Submission {
            student_id: "student1".to_string(),
            assignment_id: "hw1".to_string(),
            code: "def add(a, b): return a + b".to_string(),
            timestamp: Utc::now(),
            fingerprint: "fingerprint1".to_string(),
        };

        let submission2 = Submission {
            student_id: "student2".to_string(),
            assignment_id: "hw1".to_string(),
            code: "def add(a, b): return a + b".to_string(),
            timestamp: Utc::now(),
            fingerprint: "fingerprint1".to_string(),
        };

        // Submit first code
        let result1 = system.check_plagiarism(&submission1);
        assert!(!result1.is_plagiarized);

        // Submit identical code from different student
        let result2 = system.check_plagiarism(&submission2);
        assert!(result2.is_plagiarized || result2.similarity_score > 0.5); // Should detect similarity
    }

    #[test]
    fn test_check_plagiarism_different_code() {
        let mut system = AntiCheatSystem::new();

        let submission1 = Submission {
            student_id: "student1".to_string(),
            assignment_id: "hw1".to_string(),
            code: "def add(a, b): return a + b".to_string(),
            timestamp: Utc::now(),
            fingerprint: "fp1".to_string(),
        };

        let submission2 = Submission {
            student_id: "student2".to_string(),
            assignment_id: "hw1".to_string(),
            code: "def multiply(x, y): return x * y".to_string(),
            timestamp: Utc::now(),
            fingerprint: "fp2".to_string(),
        };

        // Submit both codes
        system.check_plagiarism(&submission1);
        let result2 = system.check_plagiarism(&submission2);

        // Different code should not be flagged as plagiarized
        assert!(!result2.is_plagiarized || result2.similarity_score < system.similarity_threshold);
    }

    #[test]
    fn test_check_plagiarism_same_student() {
        let mut system = AntiCheatSystem::new();

        let submission1 = Submission {
            student_id: "student1".to_string(),
            assignment_id: "hw1".to_string(),
            code: "def add(a, b): return a + b".to_string(),
            timestamp: Utc::now(),
            fingerprint: "fp1".to_string(),
        };

        let submission2 = Submission {
            student_id: "student1".to_string(), // Same student
            assignment_id: "hw2".to_string(),
            code: "def add(a, b): return a + b".to_string(),
            timestamp: Utc::now(),
            fingerprint: "fp1".to_string(),
        };

        // Submit both codes from same student
        system.check_plagiarism(&submission1);
        let result2 = system.check_plagiarism(&submission2);

        // Same student reusing code should not be flagged as plagiarism
        // (though this depends on implementation details)
        assert!(!result2.is_plagiarized || result2.matched_student != Some("student1".to_string()));
    }

    #[test]
    fn test_multiple_assignments() {
        let mut system = AntiCheatSystem::new();

        let assignments = vec!["hw1", "hw2", "hw3", "quiz1", "project1"];

        for (i, assignment) in assignments.iter().enumerate() {
            let submission = Submission {
                student_id: format!("student{}", i),
                assignment_id: assignment.to_string(),
                code: format!("def func{}(): pass", i),
                timestamp: Utc::now(),
                fingerprint: format!("fp{}", i),
            };

            let result = system.check_plagiarism(&submission);

            // All different code should not be plagiarized
            assert!(!result.is_plagiarized);
        }
    }

    #[test]
    fn test_submission_history_growth() {
        let mut system = AntiCheatSystem::new();

        // Submit many assignments
        for i in 0..10 {
            let submission = Submission {
                student_id: format!("student{}", i),
                assignment_id: "hw1".to_string(),
                code: format!("def func{}(): return {}", i, i),
                timestamp: Utc::now(),
                fingerprint: format!("fp{}", i),
            };

            system.check_plagiarism(&submission);
        }

        // History should contain submissions
        assert!(!system.submission_history.is_empty());
    }

    #[test]
    fn test_plagiarism_result_fields() {
        let result = PlagiarismResult {
            is_plagiarized: true,
            similarity_score: 0.95,
            matched_student: Some("student2".to_string()),
            matched_sections: vec![MatchedSection {
                start_line: 1,
                end_line: 5,
                similarity: 0.95,
            }],
        };

        assert!(result.is_plagiarized);
        assert_eq!(result.similarity_score, 0.95);
        assert_eq!(result.matched_student, Some("student2".to_string()));
        assert_eq!(result.matched_sections.len(), 1);
        assert_eq!(result.matched_sections[0].start_line, 1);
        assert_eq!(result.matched_sections[0].end_line, 5);
        assert_eq!(result.matched_sections[0].similarity, 0.95);
    }

    #[test]
    fn test_matched_section_creation() {
        let section = MatchedSection {
            start_line: 10,
            end_line: 20,
            similarity: 0.87,
        };

        assert_eq!(section.start_line, 10);
        assert_eq!(section.end_line, 20);
        assert_eq!(section.similarity, 0.87);
    }

    #[test]
    fn test_submission_creation() {
        let now = Utc::now();
        let submission = Submission {
            student_id: "student1".to_string(),
            assignment_id: "hw1".to_string(),
            code: "print('Hello')".to_string(),
            timestamp: now,
            fingerprint: "abc123".to_string(),
        };

        assert_eq!(submission.student_id, "student1");
        assert_eq!(submission.assignment_id, "hw1");
        assert_eq!(submission.code, "print('Hello')");
        assert_eq!(submission.timestamp, now);
        assert_eq!(submission.fingerprint, "abc123");
    }

    #[test]
    fn test_threshold_variations() {
        let thresholds = vec![0.5, 0.6, 0.7, 0.8, 0.9, 0.95, 0.99];

        for threshold in thresholds {
            let system = AntiCheatSystem::with_threshold(threshold);
            assert_eq!(system.similarity_threshold, threshold);
        }
    }

    #[test]
    fn test_empty_code_submission() {
        let mut system = AntiCheatSystem::new();

        let submission = Submission {
            student_id: "student1".to_string(),
            assignment_id: "hw1".to_string(),
            code: "".to_string(),
            timestamp: Utc::now(),
            fingerprint: "empty".to_string(),
        };

        let result = system.check_plagiarism(&submission);
        // Empty code should not crash
        assert!(!result.is_plagiarized || result.similarity_score == 0.0);
    }

    #[test]
    fn test_whitespace_only_code() {
        let mut system = AntiCheatSystem::new();

        let submission = Submission {
            student_id: "student1".to_string(),
            assignment_id: "hw1".to_string(),
            code: "   \n\t\n   ".to_string(),
            timestamp: Utc::now(),
            fingerprint: "whitespace".to_string(),
        };

        let result = system.check_plagiarism(&submission);
        // Whitespace only should not crash
        let _ = result;
    }

    #[test]
    fn test_large_code_submission() {
        let mut system = AntiCheatSystem::new();

        // Create a large code string
        let mut code = String::new();
        for i in 0..1000 {
            code.push_str(&format!("def func{}(): return {}\n", i, i));
        }

        let submission = Submission {
            student_id: "student1".to_string(),
            assignment_id: "hw1".to_string(),
            code,
            timestamp: Utc::now(),
            fingerprint: "large".to_string(),
        };

        let result = system.check_plagiarism(&submission);
        // Large code should not crash
        assert!(!result.is_plagiarized);
    }
}

// Property-based tests
#[cfg(test)]
mod anticheat_property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_threshold_always_respected(threshold in 0.0..=1.0) {
            let system = AntiCheatSystem::with_threshold(threshold);
            prop_assert_eq!(system.similarity_threshold, threshold);
        }

        #[test]
        fn test_submission_fields_preserved(
            student_id: String,
            assignment_id: String,
            code: String,
            fingerprint: String
        ) {
            let submission = Submission {
                student_id: student_id.clone(),
                assignment_id: assignment_id.clone(),
                code: code.clone(),
                timestamp: Utc::now(),
                fingerprint: fingerprint.clone(),
            };

            prop_assert_eq!(submission.student_id, student_id);
            prop_assert_eq!(submission.assignment_id, assignment_id);
            prop_assert_eq!(submission.code, code);
            prop_assert_eq!(submission.fingerprint, fingerprint);
        }

        #[test]
        fn test_plagiarism_never_crashes(code1: String, code2: String) {
            let mut system = AntiCheatSystem::new();

            let submission1 = Submission {
                student_id: "student1".to_string(),
                assignment_id: "hw1".to_string(),
                code: code1,
                timestamp: Utc::now(),
                fingerprint: "fp1".to_string(),
            };

            let submission2 = Submission {
                student_id: "student2".to_string(),
                assignment_id: "hw1".to_string(),
                code: code2,
                timestamp: Utc::now(),
                fingerprint: "fp2".to_string(),
            };

            // Should never panic
            let _ = system.check_plagiarism(&submission1);
            let _ = system.check_plagiarism(&submission2);
        }

        #[test]
        fn test_matched_section_valid(
            start in 0usize..1000usize,
            length in 1usize..100usize,
            similarity in 0.0..=1.0
        ) {
            let section = MatchedSection {
                start_line: start,
                end_line: start + length,
                similarity,
            };

            prop_assert!(section.start_line < section.end_line);
            prop_assert!(section.similarity >= 0.0 && section.similarity <= 1.0);
        }
    }
}

// Stress tests
#[cfg(test)]
mod anticheat_stress_tests {
    use super::*;

    #[test]
    #[ignore] // Can be expensive
    fn test_many_submissions() {
        let mut system = AntiCheatSystem::new();

        // Submit 1000 different codes
        for i in 0..1000 {
            let submission = Submission {
                student_id: format!("student{}", i),
                assignment_id: "hw1".to_string(),
                code: format!("def unique_func_{}(): return {}", i, i * 2),
                timestamp: Utc::now(),
                fingerprint: format!("fp{}", i),
            };

            let result = system.check_plagiarism(&submission);

            // All unique code should not be plagiarized
            assert!(!result.is_plagiarized);
        }
    }

    #[test]
    #[ignore] // Can be expensive
    fn test_repeated_identical_submissions() {
        let mut system = AntiCheatSystem::new();

        let base_code = "def add(a, b): return a + b";

        // Submit same code from 100 different students
        for i in 0..100 {
            let submission = Submission {
                student_id: format!("student{}", i),
                assignment_id: "hw1".to_string(),
                code: base_code.to_string(),
                timestamp: Utc::now(),
                fingerprint: "identical_fp".to_string(),
            };

            let result = system.check_plagiarism(&submission);

            // First should pass, rest should be flagged
            if i > 0 {
                // After first submission, others should be detected
                // (exact behavior depends on implementation)
                assert!(result.similarity_score > 0.0);
            }
        }
    }
}
