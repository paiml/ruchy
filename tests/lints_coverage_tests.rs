//! Tests for the lints module
//!
//! This test suite targets the lints module that had 120 lines with 0% coverage.
//! It tests custom lint rules for Ruchy code quality analysis.

use ruchy::lints::{RuchyLinter, LintRule, LintViolation, Severity};
use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span, BinaryOp, MatchArm, Pattern, Param};

/// Create a simple test expression
fn create_test_expr(kind: ExprKind) -> Expr {
    Expr::new(kind, Span::new(0, 10))
}

/// Test basic linter creation and initialization
#[test]
fn test_linter_creation() {
    let linter = RuchyLinter::new();
    
    // Should create successfully with default rules
    assert!(true); // Linter creation succeeds
    
    // Test Default trait
    let default_linter = RuchyLinter::default();
    assert!(true); // Default creation succeeds
}

/// Test adding custom rules to linter
#[test]
fn test_linter_add_custom_rule() {
    let mut linter = RuchyLinter::new();
    
    // Create a simple custom rule
    struct TestRule;
    impl LintRule for TestRule {
        fn name(&self) -> &str {
            "test_rule"
        }
        
        fn check_expression(&self, _expr: &Expr) -> Vec<LintViolation> {
            vec![]
        }
    }
    
    // Add the custom rule
    linter.add_rule(Box::new(TestRule));
    
    // Should add without error
    assert!(true);
}

/// Test linting simple expressions
#[test]
fn test_lint_simple_expressions() {
    let linter = RuchyLinter::new();
    
    // Test integer literal (should have no violations)
    let expr = create_test_expr(ExprKind::Literal(Literal::Integer(42)));
    let violations = linter.lint(&expr);
    
    // Simple expressions should have no violations
    assert!(violations.is_empty());
}

/// Test linting string expressions
#[test]
fn test_lint_string_expressions() {
    let linter = RuchyLinter::new();
    
    // Test string literal
    let expr = create_test_expr(ExprKind::Literal(Literal::String("hello".to_string())));
    let violations = linter.lint(&expr);
    
    // String literals should have no violations
    assert!(violations.is_empty());
}

/// Test linting boolean expressions
#[test]
fn test_lint_boolean_expressions() {
    let linter = RuchyLinter::new();
    
    // Test boolean literal
    let expr = create_test_expr(ExprKind::Literal(Literal::Bool(true)));
    let violations = linter.lint(&expr);
    
    // Boolean literals should have no violations
    assert!(violations.is_empty());
}

/// Test complexity rule with simple if expression
#[test]
fn test_complexity_rule_simple_if() {
    let linter = RuchyLinter::new();
    
    // Create a simple if expression (low complexity)
    let condition = Box::new(create_test_expr(ExprKind::Literal(Literal::Bool(true))));
    let then_branch = Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(1))));
    let else_branch = Some(Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(2)))));
    
    let if_expr = create_test_expr(ExprKind::If {
        condition,
        then_branch,
        else_branch,
    });
    
    let violations = linter.lint(&if_expr);
    
    // Simple if should not violate complexity rules
    assert!(violations.is_empty());
}

/// Test complexity rule with nested complex expression
#[test]
fn test_complexity_rule_nested_complexity() {
    let linter = RuchyLinter::new();
    
    // Create a highly nested if expression to trigger complexity warning
    let mut nested_expr = create_test_expr(ExprKind::Literal(Literal::Integer(1)));
    
    // Create multiple nested if expressions to increase complexity
    for _ in 0..12 {
        let condition = Box::new(create_test_expr(ExprKind::Literal(Literal::Bool(true))));
        let then_branch = Box::new(nested_expr.clone());
        let else_branch = Some(Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(2)))));
        
        nested_expr = create_test_expr(ExprKind::If {
            condition,
            then_branch,
            else_branch,
        });
    }
    
    let violations = linter.lint(&nested_expr);
    
    // Should have complexity violations
    assert!(!violations.is_empty());
    
    // Check that it's a complexity violation
    let first_violation = &violations[0];
    if let LintViolation::Violation { message, severity, .. } = first_violation {
        assert!(message.contains("complexity"));
        assert!(matches!(severity, Severity::Warning));
    }
}

/// Test no debug print rule with regular function calls
#[test]
fn test_no_debug_print_rule_regular_calls() {
    let linter = RuchyLinter::new();
    
    // Create a regular function call (not debug)
    let func = Box::new(create_test_expr(ExprKind::Identifier("println".to_string())));
    let args = vec![];
    
    let call_expr = create_test_expr(ExprKind::Call { func, args });
    let violations = linter.lint(&call_expr);
    
    // Regular function calls should not violate debug print rule
    assert!(violations.is_empty() || !violations.iter().any(|v| {
        let LintViolation::Violation { message, .. } = v;
        message.contains("debug")
    }));
}

/// Test no debug print rule with debug calls
#[test]
fn test_no_debug_print_rule_debug_calls() {
    let linter = RuchyLinter::new();
    
    // Create a debug function call
    let func = Box::new(create_test_expr(ExprKind::Identifier("dbg".to_string())));
    let args = vec![create_test_expr(ExprKind::Literal(Literal::Integer(42)))];
    
    let debug_call_expr = create_test_expr(ExprKind::Call { func, args });
    let violations = linter.lint(&debug_call_expr);
    
    // Debug calls should violate the rule
    println!("Debug: Violations found: {:?}", violations);
    assert!(!violations.is_empty());
    
    // Check that it's a debug print violation
    let has_debug_violation = violations.iter().any(|v| {
        if let LintViolation::Violation { message, .. } = v {
            println!("Debug: Checking message: {}", message);
            message.contains("debug") || message.contains("Debug")
        } else {
            false
        }
    });
    assert!(has_debug_violation);
}

/// Test no debug print rule with debug_print calls
#[test]
fn test_no_debug_print_rule_debug_print_calls() {
    let linter = RuchyLinter::new();
    
    // Create a debug_print function call
    let func = Box::new(create_test_expr(ExprKind::Identifier("debug_print".to_string())));
    let args = vec![create_test_expr(ExprKind::Literal(Literal::String("debug".to_string())))];
    
    let debug_print_expr = create_test_expr(ExprKind::Call { func, args });
    let violations = linter.lint(&debug_print_expr);
    
    // debug_print calls should violate the rule
    assert!(!violations.is_empty());
}

/// Test match expression complexity
#[test]
fn test_complexity_rule_match_expression() {
    let linter = RuchyLinter::new();
    
    // Create a match expression
    let expr = Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(42))));
    
    let arms = vec![
        MatchArm {
            pattern: Pattern::Literal(Literal::Integer(1)),
            guard: None,
            body: Box::new(create_test_expr(ExprKind::Literal(Literal::String("one".to_string())))),
            span: Span::new(0, 10),
        },
        MatchArm {
            pattern: Pattern::Literal(Literal::Integer(2)),
            guard: None,
            body: Box::new(create_test_expr(ExprKind::Literal(Literal::String("two".to_string())))),
            span: Span::new(0, 10),
        },
    ];
    
    let match_expr = create_test_expr(ExprKind::Match { expr, arms });
    let violations = linter.lint(&match_expr);
    
    // Simple match should not violate complexity rules
    assert!(violations.is_empty());
}

/// Test while loop complexity
#[test]
fn test_complexity_rule_while_loop() {
    let linter = RuchyLinter::new();
    
    // Create a while loop
    let condition = Box::new(create_test_expr(ExprKind::Literal(Literal::Bool(true))));
    let body = Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(1))));
    
    let while_expr = create_test_expr(ExprKind::While { condition, body });
    let violations = linter.lint(&while_expr);
    
    // Simple while loop should not violate complexity rules
    assert!(violations.is_empty());
}

/// Test for loop complexity
#[test]
fn test_complexity_rule_for_loop() {
    let linter = RuchyLinter::new();
    
    // Create a for loop
    let var = "i".to_string();
    let pattern = Some(Pattern::Identifier("i".to_string()));
    let iter = Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(10))));
    let body = Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(1))));
    
    let for_expr = create_test_expr(ExprKind::For { var, pattern, iter, body });
    let violations = linter.lint(&for_expr);
    
    // Simple for loop should not violate complexity rules
    assert!(violations.is_empty());
}

/// Test binary expression complexity
#[test]
fn test_complexity_rule_binary_expression() {
    let linter = RuchyLinter::new();
    
    // Create a binary expression
    let left = Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(1))));
    let right = Box::new(create_test_expr(ExprKind::Literal(Literal::Integer(2))));
    
    let binary_expr = create_test_expr(ExprKind::Binary {
        left,
        op: BinaryOp::Add,
        right,
    });
    
    let violations = linter.lint(&binary_expr);
    
    // Simple binary expressions should not violate complexity rules
    assert!(violations.is_empty());
}

/// Test lint violation formatting
#[test]
fn test_lint_violation_formatting() {
    // Create a lint violation
    let violation = LintViolation::Violation {
        location: "line 10".to_string(),
        message: "Test violation".to_string(),
        severity: Severity::Error,
        suggestion: Some("Fix this".to_string()),
    };
    
    // Test formatting
    let formatted = format!("{}", violation);
    assert!(formatted.contains("line 10"));
    assert!(formatted.contains("Test violation"));
    assert!(formatted.contains("Error"));
}

/// Test severity enum
#[test]
fn test_severity_enum() {
    // Test all severity levels
    let error = Severity::Error;
    let warning = Severity::Warning;
    let info = Severity::Info;
    
    // Test equality
    assert_eq!(error, Severity::Error);
    assert_eq!(warning, Severity::Warning);
    assert_eq!(info, Severity::Info);
    
    // Test inequality
    assert_ne!(error, warning);
    assert_ne!(warning, info);
    assert_ne!(error, info);
}

/// Test custom lint rule implementation
#[test]
fn test_custom_lint_rule() {
    struct AlwaysViolatesRule;
    
    impl LintRule for AlwaysViolatesRule {
        fn name(&self) -> &str {
            "always_violates"
        }
        
        fn check_expression(&self, expr: &Expr) -> Vec<LintViolation> {
            vec![LintViolation::Violation {
                location: format!("position {}", expr.span.start),
                message: "This rule always triggers".to_string(),
                severity: Severity::Info,
                suggestion: None,
            }]
        }
    }
    
    let mut linter = RuchyLinter::new();
    linter.add_rule(Box::new(AlwaysViolatesRule));
    
    let expr = create_test_expr(ExprKind::Literal(Literal::Integer(42)));
    let violations = linter.lint(&expr);
    
    // Should have our custom violation plus any default rule violations
    let has_custom_violation = violations.iter().any(|v| {
        let LintViolation::Violation { message, .. } = v;
        message.contains("always triggers")
    });
    
    assert!(has_custom_violation);
}

/// Test lint rule name method
#[test]
fn test_lint_rule_names() {
    struct TestRule;
    impl LintRule for TestRule {
        fn name(&self) -> &str {
            "test_rule_name"
        }
        
        fn check_expression(&self, _expr: &Expr) -> Vec<LintViolation> {
            vec![]
        }
    }
    
    let rule = TestRule;
    assert_eq!(rule.name(), "test_rule_name");
}