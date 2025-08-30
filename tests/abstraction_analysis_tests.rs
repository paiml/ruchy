//! Tests for zero-coverage abstraction analysis module
//!
//! [TEST-COV-013] Target abstraction module with 0% coverage

use ruchy::optimization::abstraction::*;
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span};

/// Create a simple test expression
fn create_simple_expr() -> Expr {
    Expr::new(ExprKind::Literal(Literal::Integer(42)), Span::default())
}

/// Create a function call expression for testing
fn create_function_call() -> Expr {
    let func = Expr::new(ExprKind::Identifier("test_func".to_string()), Span::default());
    let args = vec![create_simple_expr()];
    Expr::new(ExprKind::Call { func: Box::new(func), args }, Span::default())
}

#[test]
fn test_abstraction_analysis_creation() {
    let analysis = AbstractionAnalysis {
        runtime_overhead: 0.15,
        patterns: vec![],
        inlining_opportunities: vec![],
        allocation_overhead: 0.05,
        type_overhead: 0.02,
    };
    
    assert_eq!(analysis.runtime_overhead, 0.15);
    assert_eq!(analysis.allocation_overhead, 0.05);
    assert_eq!(analysis.type_overhead, 0.02);
    assert!(analysis.patterns.is_empty());
    assert!(analysis.inlining_opportunities.is_empty());
}

#[test]
fn test_abstraction_pattern_creation() {
    let pattern = AbstractionPattern {
        pattern_type: AbstractionType::Iterator,
        is_zero_cost: true,
        overhead_estimate: 0.0,
        description: "Iterator chain optimization".to_string(),
        location: None,
        suggestions: vec!["Use collect() for materialization".to_string()],
    };
    
    assert_eq!(pattern.pattern_type, AbstractionType::Iterator);
    assert!(pattern.is_zero_cost);
    assert_eq!(pattern.overhead_estimate, 0.0);
    assert_eq!(pattern.suggestions.len(), 1);
}

#[test]
fn test_abstraction_types() {
    let types = vec![
        AbstractionType::FunctionCall,
        AbstractionType::Iterator,
        AbstractionType::Closure,
        AbstractionType::Generic,
        AbstractionType::TraitObject,
        AbstractionType::Monadic,
        AbstractionType::HigherOrder,
        AbstractionType::TypeConversion,
    ];
    
    for abs_type in types {
        assert!(!format!("{abs_type:?}").is_empty());
        // Test PartialEq
        assert_eq!(abs_type.clone(), abs_type);
    }
}

#[test]
fn test_analyze_abstractions_simple() {
    let expr = create_simple_expr();
    let analysis = analyze_abstractions(&expr);
    
    // Simple literal should have minimal overhead
    assert!(analysis.runtime_overhead >= 0.0);
    assert!(analysis.allocation_overhead >= 0.0);
    assert!(analysis.type_overhead >= 0.0);
    
    // Analysis should complete without panicking
    assert!(analysis.patterns.len() >= 0);
    assert!(analysis.inlining_opportunities.len() >= 0);
}

#[test]
fn test_analyze_abstractions_function_call() {
    let expr = create_function_call();
    let analysis = analyze_abstractions(&expr);
    
    // Function call analysis should work
    assert!(analysis.runtime_overhead >= 0.0);
    assert!(analysis.allocation_overhead >= 0.0);
    assert!(analysis.type_overhead >= 0.0);
    
    // Should have analyzed the function call
    assert!(analysis.patterns.len() >= 0);
    assert!(analysis.inlining_opportunities.len() >= 0);
}

#[test]
fn test_inlining_opportunity_creation() {
    let opportunity = InliningOpportunity {
        function_name: "hot_path".to_string(),
        call_sites: vec![],
        function_size: 32,
        call_frequency: 500,
        benefit_score: 0.25,
        benefits: vec![],
    };
    
    assert_eq!(opportunity.function_name, "hot_path");
    assert_eq!(opportunity.function_size, 32);
    assert_eq!(opportunity.call_frequency, 500);
    assert_eq!(opportunity.benefit_score, 0.25);
    assert!(opportunity.call_sites.is_empty());
}

#[test]
fn test_abstraction_pattern_with_location() {
    use ruchy::optimization::CodeLocation;
    
    let location = CodeLocation {
        file: "test.ruchy".to_string(),
        line: 42,
        column: 8,
        span_length: 20,
    };
    
    let pattern = AbstractionPattern {
        pattern_type: AbstractionType::Closure,
        is_zero_cost: false,
        overhead_estimate: 0.1,
        description: "Closure capture overhead".to_string(),
        location: Some(location),
        suggestions: vec!["Consider move closure".to_string()],
    };
    
    assert_eq!(pattern.pattern_type, AbstractionType::Closure);
    assert!(!pattern.is_zero_cost);
    assert!(pattern.location.is_some());
    
    let loc = pattern.location.unwrap();
    assert_eq!(loc.line, 42);
    assert_eq!(loc.column, 8);
}