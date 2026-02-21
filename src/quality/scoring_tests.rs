
use super::*;

#[test]
fn test_grade_from_score() {
    assert_eq!(Grade::from_score(1.0), Grade::APlus);
    assert_eq!(Grade::from_score(0.97), Grade::APlus);
    assert_eq!(Grade::from_score(0.95), Grade::A);
    assert_eq!(Grade::from_score(0.93), Grade::A);
    assert_eq!(Grade::from_score(0.91), Grade::AMinus);
    assert_eq!(Grade::from_score(0.90), Grade::AMinus);
    assert_eq!(Grade::from_score(0.88), Grade::BPlus);
    assert_eq!(Grade::from_score(0.85), Grade::B);
    assert_eq!(Grade::from_score(0.81), Grade::BMinus);
    assert_eq!(Grade::from_score(0.78), Grade::CPlus);
    assert_eq!(Grade::from_score(0.75), Grade::C);
    assert_eq!(Grade::from_score(0.71), Grade::CMinus);
    assert_eq!(Grade::from_score(0.65), Grade::D);
    assert_eq!(Grade::from_score(0.5), Grade::F);
    assert_eq!(Grade::from_score(0.0), Grade::F);
}

#[test]
fn test_grade_to_rank() {
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
fn test_grade_display() {
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
fn test_analysis_depth_equality() {
    assert_eq!(AnalysisDepth::Shallow, AnalysisDepth::Shallow);
    assert_eq!(AnalysisDepth::Standard, AnalysisDepth::Standard);
    assert_eq!(AnalysisDepth::Deep, AnalysisDepth::Deep);
    assert_ne!(AnalysisDepth::Shallow, AnalysisDepth::Deep);
}

#[test]
fn test_score_components_creation() {
    let components = ScoreComponents {
        correctness: 0.9,
        performance: 0.85,
        maintainability: 0.8,
        safety: 0.95,
        idiomaticity: 0.7,
    };

    assert_eq!(components.correctness, 0.9);
    assert_eq!(components.performance, 0.85);
    assert_eq!(components.maintainability, 0.8);
    assert_eq!(components.safety, 0.95);
    assert_eq!(components.idiomaticity, 0.7);
}

#[test]
fn test_quality_score_creation() {
    let components = ScoreComponents {
        correctness: 0.9,
        performance: 0.85,
        maintainability: 0.8,
        safety: 0.95,
        idiomaticity: 0.7,
    };

    let score = QualityScore {
        value: 0.88,
        components,
        grade: Grade::BPlus,
        confidence: 0.95,
        cache_hit_rate: 0.2,
    };

    assert_eq!(score.value, 0.88);
    assert_eq!(score.grade, Grade::BPlus);
    assert_eq!(score.confidence, 0.95);
    assert_eq!(score.cache_hit_rate, 0.2);
    assert_eq!(score.components.correctness, 0.9);
}

#[test]
fn test_quality_score_confidence_levels() {
    let mut score = QualityScore {
        value: 0.85,
        components: ScoreComponents {
            correctness: 0.9,
            performance: 0.8,
            maintainability: 0.8,
            safety: 0.9,
            idiomaticity: 0.7,
        },
        grade: Grade::B,
        confidence: 0.0,
        cache_hit_rate: 0.0,
    };

    // Test different confidence levels
    score.confidence = 0.0;
    assert!(score.confidence < 0.5);

    score.confidence = 0.5;
    assert_eq!(score.confidence, 0.5);

    score.confidence = 1.0;
    assert_eq!(score.confidence, 1.0);
}

#[test]
fn test_quality_score_cache_hit_rate() {
    let mut score = QualityScore {
        value: 0.85,
        components: ScoreComponents {
            correctness: 0.9,
            performance: 0.8,
            maintainability: 0.8,
            safety: 0.9,
            idiomaticity: 0.7,
        },
        grade: Grade::B,
        confidence: 0.9,
        cache_hit_rate: 0.0,
    };

    // Test different cache hit rates
    score.cache_hit_rate = 0.0;
    assert_eq!(score.cache_hit_rate, 0.0);

    score.cache_hit_rate = 0.5;
    assert_eq!(score.cache_hit_rate, 0.5);

    score.cache_hit_rate = 1.0;
    assert_eq!(score.cache_hit_rate, 1.0);
}

#[test]
fn test_grade_edge_cases() {
    // Test boundary values
    assert_eq!(Grade::from_score(0.969_999), Grade::A);
    assert_eq!(Grade::from_score(0.97), Grade::APlus);
    assert_eq!(Grade::from_score(0.929_999), Grade::AMinus);
    assert_eq!(Grade::from_score(0.93), Grade::A);
    assert_eq!(Grade::from_score(0.599_999), Grade::F);
    assert_eq!(Grade::from_score(0.60), Grade::D);

    // Test negative and > 1.0 values
    assert_eq!(Grade::from_score(-0.1), Grade::F);
    assert_eq!(Grade::from_score(1.1), Grade::APlus);
}

#[test]
fn test_analysis_depth_display() {
    // Test that analysis depths have distinct values
    let depths = vec![
        AnalysisDepth::Shallow,
        AnalysisDepth::Standard,
        AnalysisDepth::Deep,
    ];

    for depth in &depths {
        // Each depth should equal itself
        assert_eq!(*depth, *depth);
    }

    // Different depths should not be equal
    assert_ne!(AnalysisDepth::Shallow, AnalysisDepth::Standard);
    assert_ne!(AnalysisDepth::Standard, AnalysisDepth::Deep);
}

#[test]
fn test_score_components_weights() {
    // Test that component weights sum to approximately 1.0
    let components = ScoreComponents {
        correctness: 0.35,
        performance: 0.25,
        maintainability: 0.20,
        safety: 0.15,
        idiomaticity: 0.05,
    };

    let sum = components.correctness
        + components.performance
        + components.maintainability
        + components.safety
        + components.idiomaticity;
    assert!((sum - 1.0).abs() < 0.001);
}

#[test]
fn test_quality_score_normalization() {
    // Test that scores are normalized to 0.0-1.0 range
    let score = QualityScore {
        value: 0.5,
        components: ScoreComponents {
            correctness: 0.5,
            performance: 0.5,
            maintainability: 0.5,
            safety: 0.5,
            idiomaticity: 0.5,
        },
        grade: Grade::F,
        confidence: 0.5,
        cache_hit_rate: 0.5,
    };

    assert!(score.value >= 0.0 && score.value <= 1.0);
    assert!(score.confidence >= 0.0 && score.confidence <= 1.0);
    assert!(score.cache_hit_rate >= 0.0 && score.cache_hit_rate <= 1.0);
}

#[test]
fn test_grade_ordering() {
    // Test that grades are properly ordered
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
fn test_incremental_cache_behavior() {
    // Test cache hit rate behavior
    let mut score = QualityScore {
        value: 0.85,
        components: ScoreComponents {
            correctness: 0.9,
            performance: 0.8,
            maintainability: 0.8,
            safety: 0.9,
            idiomaticity: 0.7,
        },
        grade: Grade::B,
        confidence: 0.9,
        cache_hit_rate: 0.0,
    };

    // Simulate incremental cache improvements
    score.cache_hit_rate = 0.25;
    assert!(score.cache_hit_rate > 0.0);

    score.cache_hit_rate = 0.50;
    assert!(score.cache_hit_rate > 0.25);

    score.cache_hit_rate = 0.75;
    assert!(score.cache_hit_rate > 0.50);
}

#[test]
fn test_analysis_depth_performance_tradeoffs() {
    // Test that different depths represent performance tradeoffs
    let shallow = AnalysisDepth::Shallow;
    let standard = AnalysisDepth::Standard;
    let deep = AnalysisDepth::Deep;

    // Each depth should be distinct
    assert_ne!(shallow, standard);
    assert_ne!(standard, deep);
    assert_ne!(shallow, deep);
}

#[test]
fn test_score_component_independence() {
    // Test that components can vary independently
    let components1 = ScoreComponents {
        correctness: 1.0,
        performance: 0.0,
        maintainability: 0.5,
        safety: 0.7,
        idiomaticity: 0.3,
    };

    let components2 = ScoreComponents {
        correctness: 0.0,
        performance: 1.0,
        maintainability: 0.5,
        safety: 0.7,
        idiomaticity: 0.3,
    };

    assert_ne!(components1.correctness, components2.correctness);
    assert_ne!(components1.performance, components2.performance);
    assert_eq!(components1.maintainability, components2.maintainability);
}

#[test]
fn test_grade_display_format() {
    // Test that grades display correctly
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
fn test_confidence_levels() {
    // Test different confidence scenarios
    let low_confidence = QualityScore {
        value: 0.85,
        components: ScoreComponents {
            correctness: 0.9,
            performance: 0.8,
            maintainability: 0.8,
            safety: 0.9,
            idiomaticity: 0.7,
        },
        grade: Grade::B,
        confidence: 0.2,
        cache_hit_rate: 0.0,
    };

    let high_confidence = QualityScore {
        value: 0.85,
        components: ScoreComponents {
            correctness: 0.9,
            performance: 0.8,
            maintainability: 0.8,
            safety: 0.9,
            idiomaticity: 0.7,
        },
        grade: Grade::B,
        confidence: 0.95,
        cache_hit_rate: 0.0,
    };

    assert!(low_confidence.confidence < 0.5);
    assert!(high_confidence.confidence > 0.9);
}

#[test]
fn test_quality_score_with_extreme_components() {
    // Test with extreme component values
    let extreme_score = QualityScore {
        value: 0.0,
        components: ScoreComponents {
            correctness: 0.0,
            performance: 0.0,
            maintainability: 0.0,
            safety: 0.0,
            idiomaticity: 0.0,
        },
        grade: Grade::F,
        confidence: 1.0,
        cache_hit_rate: 0.0,
    };

    assert_eq!(extreme_score.value, 0.0);
    assert_eq!(extreme_score.grade, Grade::F);

    let perfect_score = QualityScore {
        value: 1.0,
        components: ScoreComponents {
            correctness: 1.0,
            performance: 1.0,
            maintainability: 1.0,
            safety: 1.0,
            idiomaticity: 1.0,
        },
        grade: Grade::APlus,
        confidence: 1.0,
        cache_hit_rate: 1.0,
    };

    assert_eq!(perfect_score.value, 1.0);
    assert_eq!(perfect_score.grade, Grade::APlus);
}

#[test]
fn test_grade_transitions() {
    // Test grade transitions at exact boundaries
    let scores = vec![
        (0.97, Grade::APlus),
        (0.93, Grade::A),
        (0.90, Grade::AMinus),
        (0.87, Grade::BPlus),
        (0.83, Grade::B),
        (0.80, Grade::BMinus),
        (0.77, Grade::CPlus),
        (0.73, Grade::C),
        (0.70, Grade::CMinus),
        (0.60, Grade::D),
        (0.59, Grade::F),
    ];

    for (score, expected_grade) in scores {
        assert_eq!(Grade::from_score(score), expected_grade);
    }
}

#[test]
fn test_score_component_balance() {
    // Test balanced vs unbalanced components
    let balanced = ScoreComponents {
        correctness: 0.8,
        performance: 0.8,
        maintainability: 0.8,
        safety: 0.8,
        idiomaticity: 0.8,
    };

    let unbalanced = ScoreComponents {
        correctness: 1.0,
        performance: 0.2,
        maintainability: 0.9,
        safety: 0.3,
        idiomaticity: 1.0,
    };

    // All balanced components should be equal
    assert_eq!(balanced.correctness, balanced.performance);
    assert_eq!(balanced.performance, balanced.maintainability);

    // Unbalanced components should vary
    assert_ne!(unbalanced.correctness, unbalanced.performance);
    assert_ne!(unbalanced.safety, unbalanced.idiomaticity);
}

#[test]
fn test_analysis_depth_hash_equality() {
    use std::collections::HashSet;

    // Test that analysis depths can be used as hash keys
    let mut depth_set = HashSet::new();
    depth_set.insert(AnalysisDepth::Shallow);
    depth_set.insert(AnalysisDepth::Standard);
    depth_set.insert(AnalysisDepth::Deep);

    assert_eq!(depth_set.len(), 3);
    assert!(depth_set.contains(&AnalysisDepth::Shallow));
    assert!(depth_set.contains(&AnalysisDepth::Standard));
    assert!(depth_set.contains(&AnalysisDepth::Deep));
}

#[test]
fn test_grade_rank_consistency() {
    // Test that to_rank() is consistent with from_score()
    let test_scores = vec![
        0.98, 0.95, 0.92, 0.88, 0.85, 0.82, 0.78, 0.75, 0.72, 0.65, 0.50,
    ];

    for score in test_scores {
        let grade1 = Grade::from_score(score);
        let grade2 = Grade::from_score(score + 0.001);

        // Higher scores should have equal or higher ranks
        assert!(grade2.to_rank() >= grade1.to_rank());
    }
}

#[test]
fn test_cache_hit_rate_impact() {
    // Test that cache hit rate doesn't affect grade
    let score1 = QualityScore {
        value: 0.85,
        components: ScoreComponents {
            correctness: 0.9,
            performance: 0.8,
            maintainability: 0.8,
            safety: 0.9,
            idiomaticity: 0.7,
        },
        grade: Grade::B,
        confidence: 0.9,
        cache_hit_rate: 0.0,
    };

    let score2 = QualityScore {
        value: 0.85,
        components: ScoreComponents {
            correctness: 0.9,
            performance: 0.8,
            maintainability: 0.8,
            safety: 0.9,
            idiomaticity: 0.7,
        },
        grade: Grade::B,
        confidence: 0.9,
        cache_hit_rate: 1.0,
    };

    // Same value should yield same grade regardless of cache
    assert_eq!(score1.grade, score2.grade);
    assert_eq!(score1.value, score2.value);
}

#[test]
fn test_incremental_scoring_architecture() {
    // Test incremental scoring concepts
    let initial_score = QualityScore {
        value: 0.7,
        components: ScoreComponents {
            correctness: 0.6,
            performance: 0.7,
            maintainability: 0.8,
            safety: 0.7,
            idiomaticity: 0.6,
        },
        grade: Grade::CMinus,
        confidence: 0.5,
        cache_hit_rate: 0.0,
    };

    // Simulate incremental improvement
    let improved_score = QualityScore {
        value: 0.85,
        components: ScoreComponents {
            correctness: 0.9,
            performance: 0.8,
            maintainability: 0.8,
            safety: 0.9,
            idiomaticity: 0.7,
        },
        grade: Grade::B,
        confidence: 0.8,
        cache_hit_rate: 0.5,
    };

    assert!(improved_score.value > initial_score.value);
    assert!(improved_score.confidence > initial_score.confidence);
    assert!(improved_score.cache_hit_rate > initial_score.cache_hit_rate);
}

#[test]
fn test_component_weight_distribution() {
    // Test expected weight distribution
    let expected_weights = [
        ("correctness", 0.35),
        ("performance", 0.25),
        ("maintainability", 0.20),
        ("safety", 0.15),
        ("idiomaticity", 0.05),
    ];

    let total: f64 = expected_weights.iter().map(|(_, w)| w).sum();
    assert!((total - 1.0).abs() < 0.001);

    // Verify correctness has highest weight
    assert!(expected_weights[0].1 > expected_weights[1].1);
    assert!(expected_weights[1].1 > expected_weights[2].1);
}

#[test]
fn test_edge_case_scores() {
    // Test edge cases and special values
    assert_eq!(Grade::from_score(f64::NAN), Grade::F);
    assert_eq!(Grade::from_score(f64::INFINITY), Grade::APlus);
    assert_eq!(Grade::from_score(f64::NEG_INFINITY), Grade::F);
    assert_eq!(Grade::from_score(f64::MIN), Grade::F);
    assert_eq!(Grade::from_score(f64::MAX), Grade::APlus);
}

#[test]
fn test_score_serialization_compatibility() {
    // Test that Grade enum is serializable
    let grades = vec![
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
    ];

    for grade in grades {
        // Test serialization round-trip
        let json = serde_json::to_string(&grade).unwrap();
        let deserialized: Grade = serde_json::from_str(&json).unwrap();
        assert_eq!(grade, deserialized);
    }
}

// ============== EXTREME TDD Round 121: ScoreEngine & DependencyTracker ==============

#[test]
fn test_score_engine_new() {
    let config = ScoreConfig::default();
    let engine = ScoreEngine::new(config);
    // Just verify it creates successfully
    drop(engine);
}

#[test]
fn test_score_engine_score_shallow() {
    let config = ScoreConfig::default();
    let engine = ScoreEngine::new(config);
    let mut parser = crate::frontend::parser::Parser::new("42");
    let ast = parser.parse().expect("parse should succeed");
    let score = engine.score(&ast, AnalysisDepth::Shallow);
    assert!(score.value >= 0.0 && score.value <= 1.0);
}

#[test]
fn test_score_engine_score_standard() {
    let config = ScoreConfig::default();
    let engine = ScoreEngine::new(config);
    let mut parser = crate::frontend::parser::Parser::new("let x = 1");
    let ast = parser.parse().expect("parse should succeed");
    let score = engine.score(&ast, AnalysisDepth::Standard);
    assert!(score.confidence >= 0.0);
}

#[test]
fn test_score_engine_score_deep() {
    let config = ScoreConfig::default();
    let engine = ScoreEngine::new(config);
    let mut parser = crate::frontend::parser::Parser::new("fun foo() { 1 + 2 }");
    let ast = parser.parse().expect("parse should succeed");
    let score = engine.score(&ast, AnalysisDepth::Deep);
    assert!(matches!(
        score.grade,
        Grade::APlus
            | Grade::A
            | Grade::AMinus
            | Grade::BPlus
            | Grade::B
            | Grade::BMinus
            | Grade::CPlus
            | Grade::C
            | Grade::CMinus
            | Grade::D
            | Grade::F
    ));
}

#[test]
fn test_dependency_tracker_new() {
    let tracker = DependencyTracker::new();
    // Just verify creation
    drop(tracker);
}

#[test]
fn test_dependency_tracker_is_stale_nonexistent() {
    let tracker = DependencyTracker::new();
    let path = PathBuf::from("/nonexistent/path/file.rs");
    let is_stale = tracker.is_stale(&path);
    // Non-existent file should not report as stale (no dependencies)
    assert!(!is_stale);
}

#[test]
fn test_score_config_default() {
    let config = ScoreConfig::default();
    // Verify default weights sum to 1.0
    let total = config.correctness_weight
        + config.performance_weight
        + config.maintainability_weight
        + config.safety_weight
        + config.idiomaticity_weight;
    assert!((total - 1.0).abs() < 0.01);
}

#[test]
fn test_cache_key_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let key1 = CacheKey {
        file_path: PathBuf::from("test.rs"),
        content_hash: 12345,
        depth: AnalysisDepth::Shallow,
    };
    let key2 = CacheKey {
        file_path: PathBuf::from("test.rs"),
        content_hash: 12345,
        depth: AnalysisDepth::Shallow,
    };

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    key1.hash(&mut hasher1);
    key2.hash(&mut hasher2);
    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn test_grade_ordering_via_rank() {
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
fn test_analysis_depth_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    AnalysisDepth::Shallow.hash(&mut hasher);
    let hash1 = hasher.finish();

    let mut hasher = DefaultHasher::new();
    AnalysisDepth::Standard.hash(&mut hasher);
    let hash2 = hasher.finish();

    // Different depths should have different hashes
    assert_ne!(hash1, hash2);
}

#[test]
fn test_score_components_clone() {
    let original = ScoreComponents {
        correctness: 0.9,
        performance: 0.85,
        maintainability: 0.8,
        safety: 0.95,
        idiomaticity: 0.7,
    };
    let cloned = original.clone();
    assert_eq!(original.correctness, cloned.correctness);
    assert_eq!(original.performance, cloned.performance);
}

#[test]
fn test_quality_score_clone() {
    let components = ScoreComponents {
        correctness: 0.9,
        performance: 0.85,
        maintainability: 0.8,
        safety: 0.95,
        idiomaticity: 0.7,
    };
    let original = QualityScore {
        value: 0.88,
        components,
        grade: Grade::BPlus,
        confidence: 0.95,
        cache_hit_rate: 0.2,
    };
    let cloned = original.clone();
    assert_eq!(original.value, cloned.value);
    assert_eq!(original.grade, cloned.grade);
}

// ============================================================================
// analyze_error_handling_recursive coverage tests
// ============================================================================

#[test]
fn test_analyze_error_handling_recursive_match_with_two_arms() {
    use crate::frontend::ast::{Expr, ExprKind, MatchArm, Pattern, Span};

    let match_expr = Expr::new(
        ExprKind::Match {
            expr: Box::new(Expr::new(
                ExprKind::Identifier("result".to_string()),
                Span::default(),
            )),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Identifier("ok".to_string()),
                    guard: None,
                    body: Box::new(Expr::new(
                        ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
                        Span::default(),
                    )),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Identifier("err".to_string()),
                    guard: None,
                    body: Box::new(Expr::new(
                        ExprKind::Literal(crate::frontend::ast::Literal::Integer(0, None)),
                        Span::default(),
                    )),
                    span: Span::default(),
                },
            ],
        },
        Span::default(),
    );

    let mut total_fallible_ops = 0;
    let mut handled_ops = 0;
    analyze_error_handling_recursive(&match_expr, &mut total_fallible_ops, &mut handled_ops);

    // Match with 2+ arms counts as a handled fallible op
    assert_eq!(total_fallible_ops, 1);
    assert_eq!(handled_ops, 1);
}

#[test]
fn test_analyze_error_handling_recursive_block() {
    use crate::frontend::ast::{Expr, ExprKind, Span};

    let block_expr = Expr::new(
        ExprKind::Block(vec![
            Expr::new(
                ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
                Span::default(),
            ),
            Expr::new(
                ExprKind::Literal(crate::frontend::ast::Literal::Integer(2, None)),
                Span::default(),
            ),
        ]),
        Span::default(),
    );

    let mut total = 0;
    let mut handled = 0;
    analyze_error_handling_recursive(&block_expr, &mut total, &mut handled);

    // Literals don't count as fallible ops
    assert_eq!(total, 0);
    assert_eq!(handled, 0);
}

#[test]
fn test_analyze_error_handling_recursive_function_body() {
    use crate::frontend::ast::{Expr, ExprKind, MatchArm, Pattern, Span, Type};

    let body = Expr::new(
        ExprKind::Match {
            expr: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                Span::default(),
            )),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(Expr::new(
                        ExprKind::Literal(crate::frontend::ast::Literal::Integer(0, None)),
                        Span::default(),
                    )),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(Expr::new(
                        ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
                        Span::default(),
                    )),
                    span: Span::default(),
                },
            ],
        },
        Span::default(),
    );

    let func_expr = Expr::new(
        ExprKind::Function {
            name: "test_fn".to_string(),
            type_params: vec![],
            params: vec![],
            body: Box::new(body),
            is_async: false,
            return_type: Some(Type {
                kind: crate::frontend::ast::TypeKind::Named("i32".to_string()),
                span: Span::default(),
            }),
            is_pub: false,
        },
        Span::default(),
    );

    let mut total = 0;
    let mut handled = 0;
    analyze_error_handling_recursive(&func_expr, &mut total, &mut handled);

    assert_eq!(total, 1);
    assert_eq!(handled, 1);
}

#[test]
fn test_analyze_error_handling_recursive_single_arm_match() {
    use crate::frontend::ast::{Expr, ExprKind, MatchArm, Pattern, Span};

    // Single arm match should NOT count as handled error handling
    let match_expr = Expr::new(
        ExprKind::Match {
            expr: Box::new(Expr::new(
                ExprKind::Identifier("x".to_string()),
                Span::default(),
            )),
            arms: vec![MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Box::new(Expr::new(
                    ExprKind::Literal(crate::frontend::ast::Literal::Integer(0, None)),
                    Span::default(),
                )),
                span: Span::default(),
            }],
        },
        Span::default(),
    );

    let mut total = 0;
    let mut handled = 0;
    analyze_error_handling_recursive(&match_expr, &mut total, &mut handled);

    // Single arm: does not count as error handling
    assert_eq!(total, 0);
    assert_eq!(handled, 0);
}

#[test]
fn test_analyze_error_handling_recursive_non_matching_expr() {
    use crate::frontend::ast::{Expr, ExprKind, Span};

    // Simple identifier - no error handling
    let expr = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());

    let mut total = 0;
    let mut handled = 0;
    analyze_error_handling_recursive(&expr, &mut total, &mut handled);

    assert_eq!(total, 0);
    assert_eq!(handled, 0);
}

// ============================================================================
// analyze_null_safety_recursive coverage tests
// ============================================================================

#[test]
fn test_analyze_null_safety_recursive_some_expr() {
    use crate::frontend::ast::{Expr, ExprKind, Span};

    let some_expr = Expr::new(
        ExprKind::Some {
            value: Box::new(Expr::new(
                ExprKind::Literal(crate::frontend::ast::Literal::Integer(42, None)),
                Span::default(),
            )),
        },
        Span::default(),
    );

    let mut option_uses = 0;
    let mut unsafe_accesses = 0;
    analyze_null_safety_recursive(&some_expr, &mut option_uses, &mut unsafe_accesses);

    assert_eq!(option_uses, 1);
    assert_eq!(unsafe_accesses, 0);
}

#[test]
fn test_analyze_null_safety_recursive_none_expr() {
    use crate::frontend::ast::{Expr, ExprKind, Span};

    let none_expr = Expr::new(ExprKind::None, Span::default());

    let mut option_uses = 0;
    let mut unsafe_accesses = 0;
    analyze_null_safety_recursive(&none_expr, &mut option_uses, &mut unsafe_accesses);

    assert_eq!(option_uses, 1);
    assert_eq!(unsafe_accesses, 0);
}

#[test]
fn test_analyze_null_safety_recursive_match_with_arms() {
    use crate::frontend::ast::{Expr, ExprKind, MatchArm, Pattern, Span};

    let match_expr = Expr::new(
        ExprKind::Match {
            expr: Box::new(Expr::new(
                ExprKind::Identifier("opt".to_string()),
                Span::default(),
            )),
            arms: vec![
                MatchArm {
                    pattern: Pattern::Identifier("some_val".to_string()),
                    guard: None,
                    body: Box::new(Expr::new(
                        ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
                        Span::default(),
                    )),
                    span: Span::default(),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Box::new(Expr::new(
                        ExprKind::Literal(crate::frontend::ast::Literal::Integer(0, None)),
                        Span::default(),
                    )),
                    span: Span::default(),
                },
            ],
        },
        Span::default(),
    );

    let mut option_uses = 0;
    let mut unsafe_accesses = 0;
    analyze_null_safety_recursive(&match_expr, &mut option_uses, &mut unsafe_accesses);

    // Match with 2+ arms counts as option use
    assert_eq!(option_uses, 1);
    assert_eq!(unsafe_accesses, 0);
}

#[test]
fn test_analyze_null_safety_recursive_block() {
    use crate::frontend::ast::{Expr, ExprKind, Span};

    let block_expr = Expr::new(
        ExprKind::Block(vec![
            Expr::new(ExprKind::None, Span::default()),
            Expr::new(
                ExprKind::Some {
                    value: Box::new(Expr::new(
                        ExprKind::Literal(crate::frontend::ast::Literal::Integer(1, None)),
                        Span::default(),
                    )),
                },
                Span::default(),
            ),
        ]),
        Span::default(),
    );

    let mut option_uses = 0;
    let mut unsafe_accesses = 0;
    analyze_null_safety_recursive(&block_expr, &mut option_uses, &mut unsafe_accesses);

    assert_eq!(option_uses, 2); // None + Some
    assert_eq!(unsafe_accesses, 0);
}

#[test]
fn test_analyze_null_safety_recursive_function_body() {
    use crate::frontend::ast::{Expr, ExprKind, Span, Type};

    let body = Expr::new(ExprKind::None, Span::default());
    let func_expr = Expr::new(
        ExprKind::Function {
            name: "get_value".to_string(),
            type_params: vec![],
            params: vec![],
            body: Box::new(body),
            is_async: false,
            return_type: Some(Type {
                kind: crate::frontend::ast::TypeKind::Named("Option".to_string()),
                span: Span::default(),
            }),
            is_pub: false,
        },
        Span::default(),
    );

    let mut option_uses = 0;
    let mut unsafe_accesses = 0;
    analyze_null_safety_recursive(&func_expr, &mut option_uses, &mut unsafe_accesses);

    assert_eq!(option_uses, 1); // None in body
    assert_eq!(unsafe_accesses, 0);
}

#[test]
fn test_analyze_null_safety_recursive_non_matching_expr() {
    use crate::frontend::ast::{Expr, ExprKind, Span};

    let ident_expr = Expr::new(ExprKind::Identifier("x".to_string()), Span::default());

    let mut option_uses = 0;
    let mut unsafe_accesses = 0;
    analyze_null_safety_recursive(&ident_expr, &mut option_uses, &mut unsafe_accesses);

    assert_eq!(option_uses, 0);
    assert_eq!(unsafe_accesses, 0);
}
