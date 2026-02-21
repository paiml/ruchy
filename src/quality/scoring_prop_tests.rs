use super::*;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // Grade::from_score always returns valid grade
    // Note: from_score expects normalized value 0.0-1.0, not 0-100
    // to_rank returns 0-10 (F=0, APlus=10)
    #[test]
    fn prop_grade_from_score_valid(value in 0.0f64..=1.0) {
        let grade = Grade::from_score(value);
        // Grade should be one of the valid variants (ranks 0-10)
        let rank = grade.to_rank();
        prop_assert!((0..=10).contains(&rank));
    }

    // Grade ranks are consistent (same score = same rank)
    #[test]
    fn prop_grade_rank_consistent(value in 0.0f64..=1.0) {
        let grade1 = Grade::from_score(value);
        let grade2 = Grade::from_score(value);
        // Same score should produce same rank
        prop_assert_eq!(grade1.to_rank(), grade2.to_rank());
    }

    // ScoreConfig default is valid
    #[test]
    fn prop_score_config_default_valid(_dummy: u8) {
        let _config = ScoreConfig::default();
        prop_assert!(true);
    }

    // ScoreEngine::new never panics
    #[test]
    fn prop_score_engine_new_never_panics(_dummy: u8) {
        let config = ScoreConfig::default();
        let _engine = ScoreEngine::new(config);
        prop_assert!(true);
    }

    // DependencyTracker::new never panics
    #[test]
    fn prop_dependency_tracker_new_never_panics(_dummy: u8) {
        let tracker = DependencyTracker::new();
        // Verify tracker works with path check
        let _is_stale = tracker.is_stale(&PathBuf::from("test.rs"));
        prop_assert!(true);
    }

    // Scoring parsed integer code works
    #[test]
    fn prop_score_parsed_integer(n in -1000i64..1000) {
        let code = format!("{n}");
        let mut parser = crate::frontend::parser::Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let score = score_correctness(&ast);
            prop_assert!((0.0..=100.0).contains(&score));
        }
    }

    // Scoring parsed let statement works
    #[test]
    fn prop_score_parsed_let(n in -100i64..100) {
        let code = format!("let x = {n}");
        let mut parser = crate::frontend::parser::Parser::new(&code);
        if let Ok(ast) = parser.parse() {
            let score = score_performance(&ast);
            prop_assert!((0.0..=100.0).contains(&score));
        }
    }

    // Scoring parsed function works
    #[test]
    fn prop_score_parsed_function(_dummy: u8) {
        let code = "fun add(a, b) { a + b }";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let score = score_maintainability(&ast);
            prop_assert!((0.0..=100.0).contains(&score));
        }
    }

    // Scoring safety is bounded
    #[test]
    fn prop_score_safety_bounded(_dummy: u8) {
        let code = "let x = 42";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let score = score_safety(&ast);
            prop_assert!((0.0..=100.0).contains(&score));
        }
    }

    // Scoring idiomaticity is bounded
    #[test]
    fn prop_score_idiomaticity_bounded(_dummy: u8) {
        let code = "true && false";
        let mut parser = crate::frontend::parser::Parser::new(code);
        if let Ok(ast) = parser.parse() {
            let score = score_idiomaticity(&ast);
            prop_assert!((0.0..=100.0).contains(&score));
        }
    }
}
