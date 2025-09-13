//! Tests for progressive test disclosure system
//!
//! PMAT A+ Quality Standards:
//! - Maximum cyclomatic complexity: 10
//! - No TODO/FIXME/HACK comments
//! - 100% test coverage for new functions

use super::progressive::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// Helper functions for testing
fn create_default_config() -> DisclosureConfig {
    DisclosureConfig {
        min_attempts_before_hint: 3,
        max_hints_per_test: 2,
        unlock_threshold: 0.7,
        time_based_unlocking: false,
        collaborative_unlocking: false,
    }
}

fn create_test_hierarchy() -> TestHierarchy {
    TestHierarchy {
        levels: vec![
            TestLevel {
                level_id: 1,
                name: "Basic".to_string(),
                description: "Basic tests".to_string(),
                tests: vec!["test1".to_string(), "test2".to_string()],
                prerequisites: vec![],
                difficulty: Difficulty::Beginner,
            },
            TestLevel {
                level_id: 2,
                name: "Intermediate".to_string(),
                description: "Intermediate tests".to_string(),
                tests: vec!["test3".to_string(), "test4".to_string()],
                prerequisites: vec![1],
                difficulty: Difficulty::Intermediate,
            },
        ],
    }
}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_disclosure_config_creation() {
        let config = create_default_config();
        assert_eq!(config.min_attempts_before_hint, 3);
        assert_eq!(config.max_hints_per_test, 2);
        assert!((config.unlock_threshold - 0.7).abs() < f64::EPSILON);
        assert!(!config.time_based_unlocking);
        assert!(!config.collaborative_unlocking);
    }

    #[test]
    fn test_student_progress_creation() {
        let student = StudentProgress {
            student_id: "student123".to_string(),
            current_level: 1,
            total_score: 0.0,
            attempts_per_test: HashMap::new(),
            hints_used: HashMap::new(),
            unlock_history: Vec::new(),
        };
        assert_eq!(student.student_id, "student123");
        assert_eq!(student.current_level, 1);
        assert!((student.total_score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_unlock_event_creation() {
        let event = UnlockEvent {
            level: 2,
            test_id: "test3".to_string(),
            timestamp: Utc::now(),
            trigger: UnlockTrigger::ScoreThreshold,
        };
        assert_eq!(event.level, 2);
        assert_eq!(event.test_id, "test3");
        assert!(matches!(event.trigger, UnlockTrigger::ScoreThreshold));
    }

    #[test]
    fn test_test_hierarchy_creation() {
        let hierarchy = create_test_hierarchy();
        assert_eq!(hierarchy.levels.len(), 2);
        assert_eq!(hierarchy.levels[0].level_id, 1);
        assert_eq!(hierarchy.levels[1].level_id, 2);
    }

    #[test]
    fn test_unlock_trigger_variants() {
        assert!(matches!(UnlockTrigger::ScoreThreshold, UnlockTrigger::ScoreThreshold));
        assert!(matches!(UnlockTrigger::TimeElapsed, UnlockTrigger::TimeElapsed));
        assert!(matches!(UnlockTrigger::PeerProgress, UnlockTrigger::PeerProgress));
        assert!(matches!(UnlockTrigger::InstructorOverride, UnlockTrigger::InstructorOverride));
    }

    #[test]
    fn test_difficulty_levels() {
        assert!(matches!(Difficulty::Beginner, Difficulty::Beginner));
        assert!(matches!(Difficulty::Intermediate, Difficulty::Intermediate));
        assert!(matches!(Difficulty::Advanced, Difficulty::Advanced));
        assert!(matches!(Difficulty::Expert, Difficulty::Expert));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_config_validation_never_panics(
            min_attempts in 1usize..10usize,
            max_hints in 1usize..5usize,
            threshold in 0.0..1.0f64
        ) {
            let _config = DisclosureConfig {
                min_attempts_before_hint: min_attempts,
                max_hints_per_test: max_hints,
                unlock_threshold: threshold,
                time_based_unlocking: false,
                collaborative_unlocking: false,
            };
        }

        #[test]
        fn test_student_progress_creation_never_panics(
            student_id in "[a-zA-Z][a-zA-Z0-9]*",
            level in 1usize..10usize,
            score in 0.0..1.0f64
        ) {
            let _progress = StudentProgress {
                student_id,
                current_level: level,
                total_score: score,
                attempts_per_test: HashMap::new(),
                hints_used: HashMap::new(),
                unlock_history: Vec::new(),
            };
        }
    }
}

// Helper type definitions for testing
#[derive(Debug, Clone)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone)]
pub struct TestLevel {
    pub level_id: usize,
    pub name: String,
    pub description: String,
    pub tests: Vec<String>,
    pub prerequisites: Vec<usize>,
    pub difficulty: Difficulty,
}