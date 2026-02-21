
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

// ============================================================================
// Coverage tests for explain_delta (scoring.rs:657, 0% coverage)
// ============================================================================

fn make_quality_score(
    value: f64,
    correctness: f64,
    performance: f64,
    maintainability: f64,
    safety: f64,
    idiomaticity: f64,
) -> QualityScore {
    QualityScore {
        value,
        components: ScoreComponents {
            correctness,
            performance,
            maintainability,
            safety,
            idiomaticity,
        },
        grade: Grade::from_score(value),
        confidence: 0.9,
        cache_hit_rate: 0.0,
    }
}

#[test]
fn test_explain_delta_no_changes() {
    let score = make_quality_score(0.85, 0.9, 0.8, 0.7, 0.85, 0.75);
    let baseline = score.clone();
    let explanation = score.explain_delta(&baseline);
    assert_eq!(explanation.delta, 0.0);
    assert!(explanation.changes.is_empty());
    assert!(explanation.tradeoffs.is_empty());
    assert!(explanation.grade_change.contains("B"));
}

#[test]
fn test_explain_delta_all_improved() {
    let baseline = make_quality_score(0.7, 0.7, 0.7, 0.7, 0.7, 0.7);
    let current = make_quality_score(0.9, 0.9, 0.9, 0.9, 0.9, 0.9);
    let explanation = current.explain_delta(&baseline);
    assert!(explanation.delta > 0.0);
    assert_eq!(explanation.changes.len(), 5);
    for change in &explanation.changes {
        assert!(change.contains('+'));
    }
}

#[test]
fn test_explain_delta_all_declined() {
    let baseline = make_quality_score(0.9, 0.9, 0.9, 0.9, 0.9, 0.9);
    let current = make_quality_score(0.7, 0.7, 0.7, 0.7, 0.7, 0.7);
    let explanation = current.explain_delta(&baseline);
    assert!(explanation.delta < 0.0);
    assert_eq!(explanation.changes.len(), 5);
    for change in &explanation.changes {
        assert!(!change.contains('+'));
    }
}

#[test]
fn test_explain_delta_performance_vs_maintainability_tradeoff() {
    // Performance up, maintainability down => tradeoff detected
    let baseline = make_quality_score(0.8, 0.8, 0.7, 0.9, 0.8, 0.8);
    let current = make_quality_score(0.8, 0.8, 0.9, 0.7, 0.8, 0.8);
    let explanation = current.explain_delta(&baseline);
    assert!(explanation
        .tradeoffs
        .iter()
        .any(|t| t.contains("Performance improved at the cost of maintainability")));
}

#[test]
fn test_explain_delta_safety_vs_performance_tradeoff() {
    // Safety up, performance down => tradeoff detected
    let baseline = make_quality_score(0.8, 0.8, 0.8, 0.8, 0.7, 0.8);
    let current = make_quality_score(0.8, 0.8, 0.6, 0.8, 0.9, 0.8);
    let explanation = current.explain_delta(&baseline);
    assert!(explanation
        .tradeoffs
        .iter()
        .any(|t| t.contains("Safety improved at the cost of performance")));
}

#[test]
fn test_explain_delta_both_tradeoffs() {
    // Both tradeoffs: perf up + maint down, safety up + perf down
    // This is contradictory (perf both up and down), so only one tradeoff fires
    // Let's test them separately are possible
    let baseline = make_quality_score(0.8, 0.8, 0.8, 0.9, 0.5, 0.8);
    let current = make_quality_score(0.8, 0.8, 0.9, 0.7, 0.8, 0.8);
    let explanation = current.explain_delta(&baseline);
    // Performance improved and maintainability declined
    assert!(explanation
        .tradeoffs
        .iter()
        .any(|t| t.contains("Performance improved")));
}

#[test]
fn test_explain_delta_grade_change_format() {
    let baseline = make_quality_score(0.5, 0.5, 0.5, 0.5, 0.5, 0.5);
    let current = make_quality_score(0.98, 0.98, 0.98, 0.98, 0.98, 0.98);
    let explanation = current.explain_delta(&baseline);
    // Grade change should show "F -> A+"
    assert!(explanation.grade_change.contains("F"));
    assert!(explanation.grade_change.contains("A+"));
}

#[test]
fn test_explain_delta_small_changes_ignored() {
    // Changes < 0.01 should not be reported
    let baseline = make_quality_score(0.8, 0.8, 0.8, 0.8, 0.8, 0.8);
    let current = make_quality_score(0.8, 0.805, 0.8, 0.8, 0.8, 0.8);
    let explanation = current.explain_delta(&baseline);
    // 0.005 change is < 0.01, should be ignored
    assert!(explanation.changes.is_empty());
}

#[test]
fn test_explain_delta_change_at_threshold() {
    // Change of exactly 0.02 (> 0.01) should be reported
    let baseline = make_quality_score(0.8, 0.8, 0.8, 0.8, 0.8, 0.8);
    let current = make_quality_score(0.82, 0.82, 0.8, 0.8, 0.8, 0.8);
    let explanation = current.explain_delta(&baseline);
    assert!(explanation
        .changes
        .iter()
        .any(|c| c.contains("Correctness")));
}

// ============================================================================
// Coverage tests for optimize_cache (scoring.rs:399, 0% coverage)
// ============================================================================

#[test]
fn test_optimize_cache_removes_old_entries() {
    let mut engine = ScoreEngine::new(ScoreConfig::default());
    // Insert a cache entry with old timestamp (>5 min ago)
    let old_time = SystemTime::now() - Duration::from_secs(600);
    let key = CacheKey {
        file_path: PathBuf::from("old_file.rs"),
        content_hash: 12345,
        depth: AnalysisDepth::Shallow,
    };
    let entry = CacheEntry {
        score: make_quality_score(0.8, 0.8, 0.8, 0.8, 0.8, 0.8),
        timestamp: old_time,
        dependencies: vec![],
    };
    engine.cache.insert(key, entry);
    assert_eq!(engine.cache.len(), 1);
    engine.optimize_cache();
    assert_eq!(engine.cache.len(), 0);
}

#[test]
fn test_optimize_cache_keeps_recent_entries() {
    let mut engine = ScoreEngine::new(ScoreConfig::default());
    let key = CacheKey {
        file_path: PathBuf::from("recent_file.rs"),
        content_hash: 12345,
        depth: AnalysisDepth::Standard,
    };
    let entry = CacheEntry {
        score: make_quality_score(0.9, 0.9, 0.9, 0.9, 0.9, 0.9),
        timestamp: SystemTime::now(),
        dependencies: vec![],
    };
    engine.cache.insert(key, entry);
    engine.optimize_cache();
    assert_eq!(engine.cache.len(), 1);
}

#[test]
fn test_optimize_cache_evicts_lru_when_over_limit() {
    let mut engine = ScoreEngine::new(ScoreConfig::default());
    // Insert 600 entries (over 500 limit)
    let base_time = SystemTime::now();
    for i in 0..600 {
        let key = CacheKey {
            file_path: PathBuf::from(format!("file_{i}.rs")),
            content_hash: i as u64,
            depth: AnalysisDepth::Shallow,
        };
        let entry = CacheEntry {
            score: make_quality_score(0.8, 0.8, 0.8, 0.8, 0.8, 0.8),
            timestamp: base_time + Duration::from_millis(i as u64),
            dependencies: vec![],
        };
        engine.cache.insert(key, entry);
    }
    assert_eq!(engine.cache.len(), 600);
    engine.optimize_cache();
    // After optimization, should be trimmed to max_entries (500)
    assert!(engine.cache.len() <= 500);
}

#[test]
fn test_optimize_cache_empty_cache_no_panic() {
    let mut engine = ScoreEngine::new(ScoreConfig::default());
    engine.optimize_cache();
    assert_eq!(engine.cache.len(), 0);
}

// ============================================================================
// Coverage tests for score_progressive (scoring.rs:365, 0% coverage)
// ============================================================================

#[test]
fn test_score_progressive_with_generous_budget() {
    let mut engine = ScoreEngine::new(ScoreConfig::default());
    let code = "42";
    let mut parser = crate::frontend::parser::Parser::new(code);
    if let Ok(ast) = parser.parse() {
        let score = engine.score_progressive(
            &ast,
            PathBuf::from("test.rs"),
            code,
            Duration::from_secs(10),
        );
        // With a 10s budget, should reach deep analysis
        assert!(score.value > 0.0);
        assert!(score.value <= 1.0);
    }
}

#[test]
fn test_score_progressive_with_tiny_budget() {
    let mut engine = ScoreEngine::new(ScoreConfig::default());
    let code = "fun f(x) { x + 1 }";
    let mut parser = crate::frontend::parser::Parser::new(code);
    if let Ok(ast) = parser.parse() {
        // Budget of 1 nanosecond - should only do shallow analysis
        let score = engine.score_progressive(
            &ast,
            PathBuf::from("tiny_budget.rs"),
            code,
            Duration::from_nanos(1),
        );
        assert!(score.value > 0.0);
        assert!(score.value <= 1.0);
    }
}

#[test]
fn test_score_progressive_returns_valid_grade() {
    let mut engine = ScoreEngine::new(ScoreConfig::default());
    let code = "let x = 10";
    let mut parser = crate::frontend::parser::Parser::new(code);
    if let Ok(ast) = parser.parse() {
        let score = engine.score_progressive(
            &ast,
            PathBuf::from("grade_test.rs"),
            code,
            Duration::from_secs(5),
        );
        // Grade should be valid
        let rank = score.grade.to_rank();
        assert!(rank <= 10);
    }
}

#[test]
fn test_score_progressive_complex_code() {
    let mut engine = ScoreEngine::new(ScoreConfig::default());
    let code = r#"
            fun fibonacci(n) {
                if n <= 1 { n } else { fibonacci(n - 1) + fibonacci(n - 2) }
            }
        "#;
    let mut parser = crate::frontend::parser::Parser::new(code);
    if let Ok(ast) = parser.parse() {
        let score = engine.score_progressive(
            &ast,
            PathBuf::from("complex.rs"),
            code,
            Duration::from_secs(5),
        );
        assert!(score.value > 0.0);
    }
}

// ============================================================================
// Coverage tests for analyze_complexity_recursive (scoring.rs:1025, 28% cov)
// ============================================================================

fn make_expr(kind: ExprKind) -> crate::frontend::ast::Expr {
    crate::frontend::ast::Expr::new(kind, crate::frontend::ast::Span::new(0, 0))
}

fn lit_int(v: i64) -> crate::frontend::ast::Expr {
    make_expr(ExprKind::Literal(crate::frontend::ast::Literal::Integer(
        v, None,
    )))
}

fn lit_bool(v: bool) -> crate::frontend::ast::Expr {
    make_expr(ExprKind::Literal(crate::frontend::ast::Literal::Bool(v)))
}

#[test]
fn test_analyze_complexity_recursive_for_loop() {
    let body = lit_int(1);
    let iter_expr = lit_int(10);
    let for_expr = make_expr(ExprKind::For {
        label: None,
        var: "i".to_string(),
        pattern: None,
        iter: Box::new(iter_expr),
        body: Box::new(body),
    });
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&for_expr, &mut nested_loops, &mut recursive_calls, 0);
    // At nesting 0, for loop does not increment nested_loops
    assert_eq!(nested_loops, 0);
}

#[test]
fn test_analyze_complexity_recursive_nested_for_loops() {
    let inner_body = lit_int(1);
    let inner_iter = lit_int(5);
    let inner_for = make_expr(ExprKind::For {
        label: None,
        var: "j".to_string(),
        pattern: None,
        iter: Box::new(inner_iter),
        body: Box::new(inner_body),
    });
    let outer_iter = lit_int(10);
    let outer_for = make_expr(ExprKind::For {
        label: None,
        var: "i".to_string(),
        pattern: None,
        iter: Box::new(outer_iter),
        body: Box::new(inner_for),
    });
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&outer_for, &mut nested_loops, &mut recursive_calls, 0);
    // Inner loop is at nesting 1, should count as nested
    assert!(nested_loops > 0);
}

#[test]
fn test_analyze_complexity_recursive_while_loop() {
    let cond = lit_bool(true);
    let body = lit_int(0);
    let while_expr = make_expr(ExprKind::While {
        label: None,
        condition: Box::new(cond),
        body: Box::new(body),
    });
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&while_expr, &mut nested_loops, &mut recursive_calls, 0);
    assert_eq!(nested_loops, 0);
}

#[test]
fn test_analyze_complexity_recursive_call_expr() {
    let func = make_expr(ExprKind::Identifier("foo".to_string()));
    let arg = lit_int(1);
    let call = make_expr(ExprKind::Call {
        func: Box::new(func),
        args: vec![arg],
    });
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&call, &mut nested_loops, &mut recursive_calls, 0);
    assert!(recursive_calls > 0);
}

#[test]
fn test_analyze_complexity_recursive_block() {
    let stmt1 = lit_int(1);
    let stmt2 = lit_int(2);
    let block = make_expr(ExprKind::Block(vec![stmt1, stmt2]));
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&block, &mut nested_loops, &mut recursive_calls, 0);
    assert_eq!(nested_loops, 0);
    assert_eq!(recursive_calls, 0);
}

#[test]
fn test_analyze_complexity_recursive_function_resets_nesting() {
    // Function body should reset nesting to 0
    let body = lit_int(42);
    let func = make_expr(ExprKind::Function {
        name: "test_fn".to_string(),
        type_params: vec![],
        params: vec![],
        return_type: None,
        body: Box::new(body),
        is_async: false,
        is_pub: false,
    });
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    // Start at nesting 5 to verify function resets to 0
    analyze_complexity_recursive(&func, &mut nested_loops, &mut recursive_calls, 5);
    assert_eq!(nested_loops, 0);
}

#[test]
fn test_analyze_complexity_recursive_if_with_else() {
    let cond = lit_bool(true);
    let then_br = lit_int(1);
    let else_br = lit_int(2);
    let if_expr = make_expr(ExprKind::If {
        condition: Box::new(cond),
        then_branch: Box::new(then_br),
        else_branch: Some(Box::new(else_br)),
    });
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&if_expr, &mut nested_loops, &mut recursive_calls, 0);
    assert_eq!(nested_loops, 0);
    assert_eq!(recursive_calls, 0);
}

#[test]
fn test_analyze_complexity_recursive_if_without_else() {
    let cond = lit_bool(true);
    let then_br = lit_int(1);
    let if_expr = make_expr(ExprKind::If {
        condition: Box::new(cond),
        then_branch: Box::new(then_br),
        else_branch: None,
    });
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&if_expr, &mut nested_loops, &mut recursive_calls, 0);
    assert_eq!(nested_loops, 0);
}

#[test]
fn test_analyze_complexity_recursive_wildcard_expr() {
    // Test the catch-all `_ => {}` branch with a literal
    let lit = lit_bool(false);
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&lit, &mut nested_loops, &mut recursive_calls, 0);
    assert_eq!(nested_loops, 0);
    assert_eq!(recursive_calls, 0);
}

#[test]
fn test_analyze_complexity_recursive_call_with_multiple_args() {
    let func = make_expr(ExprKind::Identifier("bar".to_string()));
    let arg1 = lit_int(1);
    let arg2 = lit_int(2);
    let arg3 = lit_int(3);
    let call = make_expr(ExprKind::Call {
        func: Box::new(func),
        args: vec![arg1, arg2, arg3],
    });
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&call, &mut nested_loops, &mut recursive_calls, 0);
    // One call to an identifier counts as one recursive call
    assert_eq!(recursive_calls, 1);
}

#[test]
fn test_analyze_algorithmic_complexity_via_parser() {
    // Test the full pipeline through score_performance which calls
    // analyze_algorithmic_complexity -> analyze_complexity_recursive
    let code = "for i in range(10) { for j in range(10) { i + j } }";
    let mut parser = crate::frontend::parser::Parser::new(code);
    if let Ok(ast) = parser.parse() {
        let perf = score_performance(&ast);
        // Nested loops should penalize performance
        assert!(perf <= 1.0);
    }
}

#[test]
fn test_analyze_complexity_many_recursive_calls() {
    // Build AST with >2 function calls to trigger recursive_calls > 2 penalty
    let func = make_expr(ExprKind::Identifier("recurse".to_string()));
    let call1 = make_expr(ExprKind::Call {
        func: Box::new(func.clone()),
        args: vec![],
    });
    let func2 = make_expr(ExprKind::Identifier("recurse".to_string()));
    let call2 = make_expr(ExprKind::Call {
        func: Box::new(func2),
        args: vec![],
    });
    let func3 = make_expr(ExprKind::Identifier("recurse".to_string()));
    let call3 = make_expr(ExprKind::Call {
        func: Box::new(func3),
        args: vec![],
    });
    let block = make_expr(ExprKind::Block(vec![call1, call2, call3]));
    let mut nested_loops = 0;
    let mut recursive_calls = 0;
    analyze_complexity_recursive(&block, &mut nested_loops, &mut recursive_calls, 0);
    assert!(recursive_calls >= 3);
}
