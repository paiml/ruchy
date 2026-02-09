    use super::*;

    // Grade boundary tests
    #[test]
    fn test_grade_from_score_boundaries_r162() {
        // Test exact boundaries
        assert_eq!(Grade::from_score(1.0), Grade::APlus);
        assert_eq!(Grade::from_score(0.97), Grade::APlus);
        assert_eq!(Grade::from_score(0.969), Grade::A);
        assert_eq!(Grade::from_score(0.93), Grade::A);
        assert_eq!(Grade::from_score(0.929), Grade::AMinus);
        assert_eq!(Grade::from_score(0.90), Grade::AMinus);
        assert_eq!(Grade::from_score(0.899), Grade::BPlus);
        assert_eq!(Grade::from_score(0.87), Grade::BPlus);
        assert_eq!(Grade::from_score(0.869), Grade::B);
        assert_eq!(Grade::from_score(0.83), Grade::B);
        assert_eq!(Grade::from_score(0.829), Grade::BMinus);
        assert_eq!(Grade::from_score(0.80), Grade::BMinus);
        assert_eq!(Grade::from_score(0.799), Grade::CPlus);
        assert_eq!(Grade::from_score(0.77), Grade::CPlus);
        assert_eq!(Grade::from_score(0.769), Grade::C);
        assert_eq!(Grade::from_score(0.73), Grade::C);
        assert_eq!(Grade::from_score(0.729), Grade::CMinus);
        assert_eq!(Grade::from_score(0.70), Grade::CMinus);
        assert_eq!(Grade::from_score(0.699), Grade::D);
        assert_eq!(Grade::from_score(0.60), Grade::D);
        assert_eq!(Grade::from_score(0.599), Grade::F);
        assert_eq!(Grade::from_score(0.0), Grade::F);
    }

    #[test]
    fn test_grade_to_rank_all_grades_r162() {
        assert_eq!(Grade::F.to_rank(), 0);
        assert_eq!(Grade::D.to_rank(), 1);
        assert_eq!(Grade::CMinus.to_rank(), 2);
        assert_eq!(Grade::C.to_rank(), 3);
        assert_eq!(Grade::CPlus.to_rank(), 4);
        assert_eq!(Grade::BMinus.to_rank(), 5);
        assert_eq!(Grade::B.to_rank(), 6);
        assert_eq!(Grade::BPlus.to_rank(), 7);
        assert_eq!(Grade::AMinus.to_rank(), 8);
        assert_eq!(Grade::A.to_rank(), 9);
        assert_eq!(Grade::APlus.to_rank(), 10);
    }

    #[test]
    fn test_grade_display_all_r162() {
        assert_eq!(format!("{}", Grade::APlus), "A+");
        assert_eq!(format!("{}", Grade::A), "A");
        assert_eq!(format!("{}", Grade::AMinus), "A-");
        assert_eq!(format!("{}", Grade::BPlus), "B+");
        assert_eq!(format!("{}", Grade::B), "B");
        assert_eq!(format!("{}", Grade::BMinus), "B-");
        assert_eq!(format!("{}", Grade::CPlus), "C+");
        assert_eq!(format!("{}", Grade::C), "C");
        assert_eq!(format!("{}", Grade::CMinus), "C-");
        assert_eq!(format!("{}", Grade::D), "D");
        assert_eq!(format!("{}", Grade::F), "F");
    }

    #[test]
    fn test_grade_negative_score_r162() {
        // Negative scores should map to F
        assert_eq!(Grade::from_score(-1.0), Grade::F);
        assert_eq!(Grade::from_score(-0.5), Grade::F);
        assert_eq!(Grade::from_score(-100.0), Grade::F);
    }

    #[test]
    fn test_grade_over_1_score_r162() {
        // Scores over 1.0 should map to APlus
        assert_eq!(Grade::from_score(1.1), Grade::APlus);
        assert_eq!(Grade::from_score(2.0), Grade::APlus);
        assert_eq!(Grade::from_score(100.0), Grade::APlus);
    }

    // AnalysisDepth tests
    #[test]
    fn test_analysis_depth_clone_r162() {
        let depth = AnalysisDepth::Standard;
        let cloned = depth;
        assert_eq!(depth, cloned);
    }

    #[test]
    fn test_analysis_depth_debug_r162() {
        let shallow = AnalysisDepth::Shallow;
        let standard = AnalysisDepth::Standard;
        let deep = AnalysisDepth::Deep;

        assert!(format!("{:?}", shallow).contains("Shallow"));
        assert!(format!("{:?}", standard).contains("Standard"));
        assert!(format!("{:?}", deep).contains("Deep"));
    }

    #[test]
    fn test_analysis_depth_eq_r162() {
        assert_eq!(AnalysisDepth::Shallow, AnalysisDepth::Shallow);
        assert_eq!(AnalysisDepth::Standard, AnalysisDepth::Standard);
        assert_eq!(AnalysisDepth::Deep, AnalysisDepth::Deep);
        assert_ne!(AnalysisDepth::Shallow, AnalysisDepth::Standard);
        assert_ne!(AnalysisDepth::Standard, AnalysisDepth::Deep);
        assert_ne!(AnalysisDepth::Shallow, AnalysisDepth::Deep);
    }

    #[test]
    fn test_analysis_depth_hash_r162() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(AnalysisDepth::Shallow);
        set.insert(AnalysisDepth::Standard);
        set.insert(AnalysisDepth::Deep);
        assert_eq!(set.len(), 3);
        // Inserting duplicate should not increase size
        set.insert(AnalysisDepth::Shallow);
        assert_eq!(set.len(), 3);
    }

    // ScoreComponents tests
    #[test]
    fn test_score_components_clone_r162() {
        let components = ScoreComponents {
            correctness: 0.9,
            performance: 0.8,
            maintainability: 0.7,
            safety: 0.6,
            idiomaticity: 0.5,
        };
        let cloned = components.clone();
        assert_eq!(cloned.correctness, 0.9);
        assert_eq!(cloned.performance, 0.8);
        assert_eq!(cloned.maintainability, 0.7);
        assert_eq!(cloned.safety, 0.6);
        assert_eq!(cloned.idiomaticity, 0.5);
    }

    #[test]
    fn test_score_components_debug_r162() {
        let components = ScoreComponents {
            correctness: 0.95,
            performance: 0.85,
            maintainability: 0.75,
            safety: 0.65,
            idiomaticity: 0.55,
        };
        let debug_str = format!("{:?}", components);
        assert!(debug_str.contains("ScoreComponents"));
        assert!(debug_str.contains("correctness"));
        assert!(debug_str.contains("performance"));
        assert!(debug_str.contains("maintainability"));
        assert!(debug_str.contains("safety"));
        assert!(debug_str.contains("idiomaticity"));
    }

    // QualityScore tests
    #[test]
    fn test_quality_score_clone_r162() {
        let score = QualityScore {
            value: 0.85,
            components: ScoreComponents {
                correctness: 0.9,
                performance: 0.8,
                maintainability: 0.7,
                safety: 0.85,
                idiomaticity: 0.75,
            },
            grade: Grade::B,
            confidence: 0.95,
            cache_hit_rate: 0.5,
        };
        let cloned = score.clone();
        assert_eq!(cloned.value, 0.85);
        assert_eq!(cloned.grade, Grade::B);
        assert_eq!(cloned.confidence, 0.95);
        assert_eq!(cloned.cache_hit_rate, 0.5);
    }

    #[test]
    fn test_quality_score_debug_r162() {
        let score = QualityScore {
            value: 0.75,
            components: ScoreComponents {
                correctness: 0.8,
                performance: 0.7,
                maintainability: 0.6,
                safety: 0.75,
                idiomaticity: 0.65,
            },
            grade: Grade::CPlus,
            confidence: 0.8,
            cache_hit_rate: 0.25,
        };
        let debug_str = format!("{:?}", score);
        assert!(debug_str.contains("QualityScore"));
        assert!(debug_str.contains("value"));
        assert!(debug_str.contains("grade"));
        assert!(debug_str.contains("confidence"));
    }

    // ScoreConfig tests
    #[test]
    fn test_score_config_default_r162() {
        let config = ScoreConfig::default();
        // Default should have reasonable values
        assert!(config.correctness_weight >= 0.0 && config.correctness_weight <= 1.0);
        assert!(config.performance_weight >= 0.0 && config.performance_weight <= 1.0);
        assert!(config.maintainability_weight >= 0.0 && config.maintainability_weight <= 1.0);
        assert!(config.safety_weight >= 0.0 && config.safety_weight <= 1.0);
        assert!(config.idiomaticity_weight >= 0.0 && config.idiomaticity_weight <= 1.0);
    }

    // DependencyTracker tests
    #[test]
    fn test_dependency_tracker_new_r162() {
        let tracker = DependencyTracker::new();
        // New tracker should work
        let path = PathBuf::from("nonexistent.rs");
        let _is_stale = tracker.is_stale(&path);
    }

    #[test]
    fn test_dependency_tracker_is_stale_nonexistent_r162() {
        let tracker = DependencyTracker::new();
        let path = PathBuf::from("definitely_does_not_exist_12345.rs");
        // Test that is_stale doesn't panic on nonexistent files
        let is_stale = tracker.is_stale(&path);
        // Note: Implementation returns false for files not tracked
        assert!(!is_stale);
    }

    // Grade comparison tests
    #[test]
    fn test_grade_rank_ordering_r162() {
        // Higher grades should have higher ranks
        assert!(Grade::APlus.to_rank() > Grade::A.to_rank());
        assert!(Grade::A.to_rank() > Grade::AMinus.to_rank());
        assert!(Grade::AMinus.to_rank() > Grade::BPlus.to_rank());
        assert!(Grade::BPlus.to_rank() > Grade::B.to_rank());
        assert!(Grade::B.to_rank() > Grade::BMinus.to_rank());
        assert!(Grade::BMinus.to_rank() > Grade::CPlus.to_rank());
        assert!(Grade::CPlus.to_rank() > Grade::C.to_rank());
        assert!(Grade::C.to_rank() > Grade::CMinus.to_rank());
        assert!(Grade::CMinus.to_rank() > Grade::D.to_rank());
        assert!(Grade::D.to_rank() > Grade::F.to_rank());
    }

    #[test]
    fn test_grade_serialize_deserialize_r162() {
        // Grade should be serializable
        let grade = Grade::AMinus;
        let serialized = serde_json::to_string(&grade).unwrap();
        let deserialized: Grade = serde_json::from_str(&serialized).unwrap();
        assert_eq!(grade, deserialized);
    }

    #[test]
    fn test_grade_all_serialize_r162() {
        for grade in [
            Grade::APlus,
            Grade::A,
            Grade::AMinus,
            Grade::BPlus,
            Grade::B,
            Grade::BMinus,
            Grade::CPlus,
            Grade::C,
            Grade::CMinus,
            Grade::D,
            Grade::F,
        ] {
            let serialized = serde_json::to_string(&grade).unwrap();
            let deserialized: Grade = serde_json::from_str(&serialized).unwrap();
            assert_eq!(grade, deserialized);
        }
    }

    // Edge case scoring tests
    #[test]
    fn test_score_empty_program_r162() {
        let code = "";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            // Empty program should still produce valid scores
            let correctness = score_correctness(&ast);
            let performance = score_performance(&ast);
            let maintainability = score_maintainability(&ast);
            let safety = score_safety(&ast);
            let idiomaticity = score_idiomaticity(&ast);

            assert!((0.0..=100.0).contains(&correctness));
            assert!((0.0..=100.0).contains(&performance));
            assert!((0.0..=100.0).contains(&maintainability));
            assert!((0.0..=100.0).contains(&safety));
            assert!((0.0..=100.0).contains(&idiomaticity));
        }
    }

    #[test]
    fn test_score_simple_literal_r162() {
        let code = "42";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let correctness = score_correctness(&ast);
            assert!((0.0..=100.0).contains(&correctness));
        }
    }

    #[test]
    fn test_score_string_literal_r162() {
        let code = r#""hello world""#;
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let idiomaticity = score_idiomaticity(&ast);
            assert!((0.0..=100.0).contains(&idiomaticity));
        }
    }

    #[test]
    fn test_score_boolean_literal_r162() {
        let code = "true";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let safety = score_safety(&ast);
            assert!((0.0..=100.0).contains(&safety));
        }
    }

    #[test]
    fn test_score_binary_expression_r162() {
        let code = "1 + 2 * 3";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let performance = score_performance(&ast);
            assert!((0.0..=100.0).contains(&performance));
        }
    }

    #[test]
    fn test_score_nested_functions_r162() {
        let code = "fun outer() { fun inner() { 42 } }";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let maintainability = score_maintainability(&ast);
            assert!((0.0..=100.0).contains(&maintainability));
        }
    }

    #[test]
    fn test_score_if_expression_r162() {
        let code = "if true { 1 } else { 2 }";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let correctness = score_correctness(&ast);
            assert!((0.0..=100.0).contains(&correctness));
        }
    }

    #[test]
    fn test_score_while_loop_r162() {
        let code = "while false { 1 }";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let safety = score_safety(&ast);
            assert!((0.0..=100.0).contains(&safety));
        }
    }

    #[test]
    fn test_score_array_literal_r162() {
        let code = "[1, 2, 3, 4, 5]";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let performance = score_performance(&ast);
            assert!((0.0..=100.0).contains(&performance));
        }
    }

    #[test]
    fn test_score_lambda_expression_r162() {
        let code = "|x| x * 2";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let maintainability = score_maintainability(&ast);
            assert!((0.0..=100.0).contains(&maintainability));
        }
    }

    #[test]
    fn test_score_idiomaticity_compound_expression_r162() {
        let code = "fun map_values(arr) { arr.map(|x| x * 2) }";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let idiomaticity = score_idiomaticity(&ast);
            assert!((0.0..=100.0).contains(&idiomaticity));
        }
    }
