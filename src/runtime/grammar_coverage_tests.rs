//! Comprehensive tests for Grammar Coverage module
//! Target: 100% coverage for production code analysis
//! Quality: PMAT A+ standards, ≤10 complexity per function

#[cfg(test)]
mod grammar_coverage_tests {
    use crate::runtime::grammar_coverage::{GrammarCoverageMatrix, ProductionStats};
    use crate::frontend::ast::{Expr, ExprKind, Literal, Span, Attribute};
    use anyhow::{Result, anyhow};
    use std::time::Duration;
    
    // ========== ProductionStats Tests ==========
    
    #[test]
    fn test_production_stats_default() {
        let stats = ProductionStats::default();
        assert_eq!(stats.hit_count, 0);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.avg_latency_ns, 0);
        assert!(stats.error_patterns.is_empty());
    }
    
    // ========== GrammarCoverageMatrix Tests ==========
    
    #[test]
    fn test_matrix_creation() {
        let matrix = GrammarCoverageMatrix::new();
        assert!(matrix.productions.is_empty());
        assert!(matrix.ast_variants.is_empty());
        assert!(matrix.uncovered.is_empty());
    }
    
    #[test]
    fn test_record_successful_parse() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        matrix.record(
            "literal",
            "42",
            Ok(expr),
            Duration::from_nanos(1000)
        );
        
        assert_eq!(matrix.productions.len(), 1);
        let stats = matrix.productions.get("literal").unwrap();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.success_count, 1);
        assert_eq!(stats.avg_latency_ns, 1000);
    }
    
    #[test]
    fn test_record_failed_parse() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        matrix.record(
            "binary_expr",
            "1 + ",
            Err(anyhow!("Expected expression")),
            Duration::from_nanos(500)
        );
        
        let stats = matrix.productions.get("binary_expr").unwrap();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.success_count, 0);
        assert_eq!(stats.avg_latency_ns, 500);
        assert_eq!(stats.error_patterns.len(), 1);
    }
    
    #[test]
    fn test_multiple_recordings() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Bool(true)),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        // Record multiple successful parses
        for i in 0..5 {
            matrix.record(
                "bool_literal",
                "true",
                Ok(expr.clone()),
                Duration::from_nanos(100 * (i + 1))
            );
        }
        
        let stats = matrix.productions.get("bool_literal").unwrap();
        assert_eq!(stats.hit_count, 5);
        assert_eq!(stats.success_count, 5);
        // Average of 100, 200, 300, 400, 500 = 300
        assert_eq!(stats.avg_latency_ns, 300);
    }
    
    #[test]
    fn test_coverage_percentage() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Add some uncovered productions
        matrix.uncovered = vec!["if_expr", "match_expr", "for_loop"];
        
        // Record coverage for some productions
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(1)),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        matrix.record("literal", "1", Ok(expr.clone()), Duration::from_nanos(100));
        matrix.record("binary", "1+1", Ok(expr), Duration::from_nanos(200));
        
        let percentage = coverage_percentage(&matrix);
        // 2 covered out of 5 total (2 covered + 3 uncovered) = 40%
        assert_eq!(percentage, 40.0);
    }
    
    #[test]
    fn test_get_uncovered_productions() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        matrix.uncovered = vec!["async_block", "generator", "trait_impl"];
        
        let uncovered = get_uncovered(&matrix);
        assert_eq!(uncovered.len(), 3);
        assert!(uncovered.contains(&"async_block"));
        assert!(uncovered.contains(&"generator"));
        assert!(uncovered.contains(&"trait_impl"));
    }
    
    #[test]
    fn test_ast_variant_tracking() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Create different expression types
        let int_expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(42)),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        let bool_expr = Expr {
            kind: ExprKind::Literal(Literal::Bool(true)),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        matrix.record("literal", "42", Ok(int_expr), Duration::from_nanos(100));
        matrix.record("literal", "true", Ok(bool_expr), Duration::from_nanos(100));
        
        // Should have recorded both AST variants
        assert!(matrix.ast_variants.contains("Literal"));
    }
    
    #[test]
    fn test_error_pattern_collection() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        // Record various error patterns
        matrix.record(
            "function",
            "fn(",
            Err(anyhow!("Missing parameter list")),
            Duration::from_nanos(100)
        );
        
        matrix.record(
            "function",
            "fn name",
            Err(anyhow!("Expected opening parenthesis")),
            Duration::from_nanos(100)
        );
        
        let stats = matrix.productions.get("function").unwrap();
        assert_eq!(stats.error_patterns.len(), 2);
        assert!(stats.error_patterns[0].contains("Missing parameter list"));
        assert!(stats.error_patterns[1].contains("Expected opening parenthesis"));
    }
    
    #[test]
    fn test_latency_averaging() {
        let mut matrix = GrammarCoverageMatrix::new();
        
        let expr = Expr {
            kind: ExprKind::Literal(Literal::Integer(1)),
            span: Span { start: 0, end: 0 },
            attributes: vec![],
        };
        
        // Test latency averaging algorithm
        matrix.record("test", "1", Ok(expr.clone()), Duration::from_nanos(1000));
        assert_eq!(matrix.productions.get("test").unwrap().avg_latency_ns, 1000);
        
        matrix.record("test", "2", Ok(expr.clone()), Duration::from_nanos(2000));
        assert_eq!(matrix.productions.get("test").unwrap().avg_latency_ns, 1500);
        
        matrix.record("test", "3", Ok(expr), Duration::from_nanos(3000));
        assert_eq!(matrix.productions.get("test").unwrap().avg_latency_ns, 2000);
    }
    
    // ========== Helper Functions (≤10 complexity each) ==========
    
    // Test helper functions
    fn coverage_percentage(matrix: &GrammarCoverageMatrix) -> f64 {
        let covered = matrix.productions.len();
        let total = covered + matrix.uncovered.len();
        
        if total == 0 {
            100.0
        } else {
            (covered as f64 / total as f64) * 100.0
        }
    }
    
    fn get_uncovered(matrix: &GrammarCoverageMatrix) -> Vec<&'static str> {
        matrix.uncovered.clone()
    }
    
    // ========== Property Tests ==========
    
    use proptest::prelude::*;
    
    proptest! {
        #[test]
        fn test_latency_never_negative(
            latencies in prop::collection::vec(0u64..1_000_000, 1..100)
        ) {
            let mut matrix = GrammarCoverageMatrix::new();
            let expr = Expr {
                kind: ExprKind::Literal(Literal::Integer(1)),
                span: Span { start: 0, end: 0 },
                attributes: vec![],
            };
            
            for latency in latencies {
                matrix.record(
                    "test",
                    "input",
                    Ok(expr.clone()),
                    Duration::from_nanos(latency)
                );
            }
            
            let stats = matrix.productions.get("test").unwrap();
            assert!(stats.avg_latency_ns < u64::MAX);
        }
        
        #[test]
        fn test_hit_count_consistency(
            num_records in 1usize..100
        ) {
            let mut matrix = GrammarCoverageMatrix::new();
            
            for i in 0..num_records {
                let result = if i % 2 == 0 {
                    Ok(Expr {
                        kind: ExprKind::Literal(Literal::Integer(i as i64)),
                        span: Span { start: 0, end: 0 },
                        attributes: vec![],
                    })
                } else {
                    Err(anyhow!("Test error"))
                };
                
                matrix.record(
                    "test",
                    &format!("input_{}", i),
                    result,
                    Duration::from_nanos(100)
                );
            }
            
            let stats = matrix.productions.get("test").unwrap();
            assert_eq!(stats.hit_count, num_records);
            assert!(stats.success_count <= stats.hit_count);
        }
    }
}