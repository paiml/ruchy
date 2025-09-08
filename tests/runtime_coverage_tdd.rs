//! TDD tests for runtime module coverage improvement
//! Target: Restore runtime coverage from 55% to 80%+

use ruchy::runtime::grammar_coverage::{GrammarCoverageMatrix, ProductionStats};
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};
use std::time::Duration;

#[test]
fn test_grammar_coverage_new() {
    let matrix = GrammarCoverageMatrix::new();
    assert!(matrix.productions.is_empty());
    assert!(matrix.ast_variants.is_empty());
    assert!(matrix.uncovered.is_empty());
}

#[test]
fn test_grammar_coverage_record_success() {
    let mut matrix = GrammarCoverageMatrix::new();
    
    // Create a simple expression for testing
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    };
    
    // Record a successful parse
    matrix.record(
        "integer_literal",
        "42",
        Ok(expr),
        Duration::from_nanos(1000),
    );
    
    // Verify the recording
    assert_eq!(matrix.productions.len(), 1);
    let stats = matrix.productions.get("integer_literal").unwrap();
    assert_eq!(stats.hit_count, 1);
    assert_eq!(stats.success_count, 1);
    assert_eq!(stats.avg_latency_ns, 1000);
    assert!(stats.error_patterns.is_empty());
}

#[test]
fn test_grammar_coverage_record_error() {
    use anyhow::anyhow;
    
    let mut matrix = GrammarCoverageMatrix::new();
    
    // Record a failed parse
    matrix.record(
        "invalid_syntax",
        "!!!",
        Err(anyhow!("Unexpected token")),
        Duration::from_nanos(500),
    );
    
    // Verify the recording
    assert_eq!(matrix.productions.len(), 1);
    let stats = matrix.productions.get("invalid_syntax").unwrap();
    assert_eq!(stats.hit_count, 1);
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.avg_latency_ns, 500);
    assert_eq!(stats.error_patterns.len(), 1);
    assert_eq!(stats.error_patterns[0], "Unexpected token");
}

#[test]
fn test_grammar_coverage_multiple_records() {
    let mut matrix = GrammarCoverageMatrix::new();
    
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    };
    
    // Record multiple attempts of the same production
    for i in 0..5 {
        matrix.record(
            "integer_literal",
            &format!("{}", i),
            Ok(expr.clone()),
            Duration::from_nanos(1000 * (i as u64 + 1)),
        );
    }
    
    let stats = matrix.productions.get("integer_literal").unwrap();
    assert_eq!(stats.hit_count, 5);
    assert_eq!(stats.success_count, 5);
    // Average of 1000, 2000, 3000, 4000, 5000 = 3000
    assert_eq!(stats.avg_latency_ns, 3000);
}

#[test]
fn test_grammar_coverage_get_coverage_percentage() {
    let mut matrix = GrammarCoverageMatrix::new();
    
    // Initially uncovered
    matrix.uncovered = vec!["expr", "stmt", "pattern", "type"];
    assert_eq!(matrix.get_coverage_percentage(), 0.0);
    
    // Add some coverage
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    };
    
    matrix.record("expr", "42", Ok(expr.clone()), Duration::from_nanos(100));
    matrix.record("stmt", "let x = 42", Ok(expr), Duration::from_nanos(200));
    
    // expr and stmt are now covered, so uncovered should be ["pattern", "type"]
    // 2 covered out of 4 total (2 covered + 2 uncovered) = 50%
    // But the implementation counts unique productions, not the uncovered list
    // So we need to fix the logic
    assert_eq!(matrix.get_coverage_percentage(), 50.0);
}

#[test]
fn test_grammar_coverage_report() {
    use anyhow::anyhow;
    
    let mut matrix = GrammarCoverageMatrix::new();
    matrix.uncovered = vec!["expr", "stmt", "pattern"];
    
    let expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    };
    
    // Add some data
    matrix.record("expr", "42", Ok(expr.clone()), Duration::from_nanos(1000));
    matrix.record("expr", "43", Ok(expr), Duration::from_nanos(2000));
    matrix.record("stmt", "invalid", Err(anyhow!("Parse error")), Duration::from_nanos(500));
    
    let report = matrix.generate_report();
    
    // Report should contain coverage percentage
    assert!(report.contains("Coverage"));
    // Report should show production stats
    assert!(report.contains("expr"));
    assert!(report.contains("stmt"));
    // Report should show uncovered productions
    assert!(report.contains("pattern"));
}

#[test]
fn test_grammar_coverage_ast_variants() {
    let mut matrix = GrammarCoverageMatrix::new();
    
    // Track different AST variants
    let int_expr = Expr {
        kind: ExprKind::Literal(Literal::Integer(42)),
        span: Span::default(),
        attributes: vec![],
    };
    
    let float_expr = Expr {
        kind: ExprKind::Literal(Literal::Float(3.14)),
        span: Span::default(),
        attributes: vec![],
    };
    
    matrix.record("literal", "42", Ok(int_expr), Duration::from_nanos(100));
    matrix.record("literal", "3.14", Ok(float_expr), Duration::from_nanos(100));
    
    // AST variants should be tracked (both are Literal variants)
    assert_eq!(matrix.ast_variants.len(), 1);
    assert!(matrix.ast_variants.contains("Literal"));
}

#[test]
fn test_production_stats_default() {
    let stats = ProductionStats::default();
    assert_eq!(stats.hit_count, 0);
    assert_eq!(stats.success_count, 0);
    assert_eq!(stats.avg_latency_ns, 0);
    assert!(stats.error_patterns.is_empty());
}

#[test]
fn test_grammar_coverage_error_deduplication() {
    use anyhow::anyhow;
    
    let mut matrix = GrammarCoverageMatrix::new();
    
    // Record the same error multiple times
    for _ in 0..5 {
        matrix.record(
            "invalid",
            "!!!",
            Err(anyhow!("Unexpected token")),
            Duration::from_nanos(100),
        );
    }
    
    let stats = matrix.productions.get("invalid").unwrap();
    assert_eq!(stats.hit_count, 5);
    assert_eq!(stats.success_count, 0);
    // Error should only be recorded once (deduplication)
    assert_eq!(stats.error_patterns.len(), 1);
}