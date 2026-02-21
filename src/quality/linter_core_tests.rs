use super::*;
use crate::frontend::ast::{
    BinaryOp, Expr, ExprKind, Literal, MatchArm, Param, Pattern, Span, StringPart,
    StructPatternField, Type, TypeKind,
};
// Helper functions for consistent test setup
fn create_test_span() -> Span {
    Span { start: 0, end: 1 }
}
fn create_test_linter() -> Linter {
    Linter::new()
}
fn create_test_linter_with_rules(rules: &str) -> Linter {
    let mut linter = Linter::new();
    linter.set_rules(rules);
    linter
}
fn create_test_expr_literal_int(value: i64) -> Expr {
    Expr::new(
        ExprKind::Literal(Literal::Integer(value, None)),
        create_test_span(),
    )
}
fn create_test_expr_identifier(name: &str) -> Expr {
    Expr::new(ExprKind::Identifier(name.to_string()), create_test_span())
}
fn create_test_expr_let(name: &str, value: Expr, body: Expr) -> Expr {
    Expr::new(
        ExprKind::Let {
            name: name.to_string(),
            type_annotation: None,
            value: Box::new(value),
            body: Box::new(body),
            is_mutable: false,
            else_block: None,
        },
        create_test_span(),
    )
}
fn create_test_expr_function(name: &str, params: Vec<Param>, body: Expr) -> Expr {
    Expr::new(
        ExprKind::Function {
            name: name.to_string(),
            type_params: vec![],
            params,
            return_type: None,
            body: Box::new(body),
            is_async: false,
            is_pub: false,
        },
        create_test_span(),
    )
}
fn create_test_param(name: &str) -> Param {
    Param {
        pattern: Pattern::Identifier(name.to_string()),
        ty: Type {
            kind: TypeKind::Named("Any".to_string()),
            span: create_test_span(),
        },
        span: create_test_span(),
        is_mutable: false,
        default_value: None,
    }
}
fn create_test_expr_block(exprs: Vec<Expr>) -> Expr {
    Expr::new(ExprKind::Block(exprs), create_test_span())
}
fn create_test_expr_binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::new(
        ExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        },
        create_test_span(),
    )
}
fn create_test_expr_call(func: Expr, args: Vec<Expr>) -> Expr {
    Expr::new(
        ExprKind::Call {
            func: Box::new(func),
            args,
        },
        create_test_span(),
    )
}
fn create_test_expr_if(condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Expr {
    Expr::new(
        ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        },
        create_test_span(),
    )
}
fn create_test_expr_for(var: &str, pattern: Option<Pattern>, iter: Expr, body: Expr) -> Expr {
    Expr::new(
        ExprKind::For {
            label: None,
            var: var.to_string(),
            pattern,
            iter: Box::new(iter),
            body: Box::new(body),
        },
        create_test_span(),
    )
}
fn create_test_expr_match(expr: Expr, arms: Vec<MatchArm>) -> Expr {
    Expr::new(
        ExprKind::Match {
            expr: Box::new(expr),
            arms,
        },
        create_test_span(),
    )
}
fn create_test_match_arm(pattern: Pattern, body: Expr) -> MatchArm {
    MatchArm {
        pattern,
        guard: None,
        body: Box::new(body),
        span: create_test_span(),
    }
}
fn create_test_expr_lambda(params: Vec<Param>, body: Expr) -> Expr {
    Expr::new(
        ExprKind::Lambda {
            params,
            body: Box::new(body),
        },
        create_test_span(),
    )
}
fn create_test_expr_method_call(receiver: Expr, method: &str, args: Vec<Expr>) -> Expr {
    Expr::new(
        ExprKind::MethodCall {
            receiver: Box::new(receiver),
            method: method.to_string(),
            args,
        },
        create_test_span(),
    )
}
fn create_test_expr_while(condition: Expr, body: Expr) -> Expr {
    Expr::new(
        ExprKind::While {
            label: None,
            condition: Box::new(condition),
            body: Box::new(body),
        },
        create_test_span(),
    )
}
fn create_test_expr_return(value: Option<Expr>) -> Expr {
    Expr::new(
        ExprKind::Return {
            value: value.map(Box::new),
        },
        create_test_span(),
    )
}
// ========== Linter Construction Tests ==========
#[test]
fn test_linter_creation() {
    let linter = Linter::new();
    assert_eq!(linter.rules.len(), 8); // Default rules count
    assert!(!linter.strict_mode);
    assert_eq!(linter.max_complexity, 10);
}
#[test]
fn test_linter_default() {
    let linter = Linter::default();
    assert_eq!(linter.rules.len(), 8);
    assert!(!linter.strict_mode);
    assert_eq!(linter.max_complexity, 10);
}
#[test]
fn test_linter_set_strict_mode() {
    let mut linter = Linter::new();
    linter.set_strict_mode(true);
    assert!(linter.strict_mode);
}
// ========== Rule Configuration Tests ==========
#[test]
fn test_set_rules_unused() {
    let mut linter = Linter::new();
    linter.set_rules("unused");
    assert_eq!(linter.rules.len(), 4); // UnusedVariable, Parameter, LoopVariable, MatchBinding
}
#[test]
fn test_set_rules_undefined() {
    let mut linter = Linter::new();
    linter.set_rules("undefined");
    assert_eq!(linter.rules.len(), 1);
    assert!(matches!(linter.rules[0], LintRule::UndefinedVariable));
}
#[test]
fn test_set_rules_shadowing() {
    let mut linter = Linter::new();
    linter.set_rules("shadowing");
    assert_eq!(linter.rules.len(), 1);
    assert!(matches!(linter.rules[0], LintRule::VariableShadowing));
}
#[test]
fn test_set_rules_complexity() {
    let mut linter = Linter::new();
    linter.set_rules("complexity");
    assert_eq!(linter.rules.len(), 1);
    assert!(matches!(linter.rules[0], LintRule::ComplexityLimit));
}
#[test]
fn test_set_rules_multiple() {
    let mut linter = Linter::new();
    linter.set_rules("undefined,shadowing,complexity");
    assert_eq!(linter.rules.len(), 3);
}
#[test]
fn test_set_rules_unknown() {
    let mut linter = Linter::new();
    linter.set_rules("unknown_rule");
    assert_eq!(linter.rules.len(), 0);
}
#[test]
fn test_set_rules_style_security_performance() {
    let mut linter = Linter::new();
    linter.set_rules("style,security,performance");
    assert_eq!(linter.rules.len(), 3);
    assert!(linter
        .rules
        .iter()
        .any(|r| matches!(r, LintRule::StyleViolation)));
    assert!(linter.rules.iter().any(|r| matches!(r, LintRule::Security)));
    assert!(linter
        .rules
        .iter()
        .any(|r| matches!(r, LintRule::Performance)));
}
// ========== Scope Tests ==========
#[test]
fn test_scope_creation() {
    let scope = Scope::new();
    assert!(scope.variables.is_empty());
    assert!(scope.parent.is_none());
}
#[test]
fn test_scope_with_parent() {
    let parent_scope = Scope::new();
    let child_scope = Scope::with_parent(parent_scope);
    assert!(child_scope.parent.is_some());
}
#[test]
fn test_scope_define_variable() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), 1, 1, VarType::Local);
    assert!(scope.variables.contains_key("x"));
    assert!(!scope.variables["x"].used);
}
#[test]
fn test_scope_mark_used() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), 1, 1, VarType::Local);
    assert!(scope.mark_used("x"));
    assert!(scope.variables["x"].used);
}
#[test]
fn test_scope_mark_used_undefined() {
    let mut scope = Scope::new();
    assert!(!scope.mark_used("undefined_var"));
}
#[test]
fn test_scope_mark_used_in_parent() {
    let mut parent_scope = Scope::new();
    parent_scope.define("x".to_string(), 1, 1, VarType::Local);
    let mut child_scope = Scope::with_parent(parent_scope);
    assert!(child_scope.mark_used("x"));
}
#[test]
fn test_scope_is_defined() {
    let mut scope = Scope::new();
    scope.define("x".to_string(), 1, 1, VarType::Local);
    assert!(scope.is_defined("x"));
    assert!(!scope.is_defined("y"));
}
#[test]
fn test_scope_is_defined_in_parent() {
    let mut parent_scope = Scope::new();
    parent_scope.define("x".to_string(), 1, 1, VarType::Local);
    let child_scope = Scope::with_parent(parent_scope);
    assert!(child_scope.is_defined("x"));
}
#[test]
fn test_scope_is_shadowing() {
    let mut parent_scope = Scope::new();
    parent_scope.define("x".to_string(), 1, 1, VarType::Local);
    let child_scope = Scope::with_parent(parent_scope);
    assert!(child_scope.is_shadowing("x"));
    assert!(!child_scope.is_shadowing("y"));
}
// ========== Lint Issue Tests ==========
#[test]
fn test_lint_issue_serialization() {
    let issue = LintIssue {
        line: 5,
        column: 10,
        severity: "warning".to_string(),
        rule: "unused_variable".to_string(),
        message: "unused variable: x".to_string(),
        suggestion: "Remove unused variable 'x'".to_string(),
        issue_type: "unused_variable".to_string(),
        name: "x".to_string(),
    };
    let json = serde_json::to_string(&issue);
    assert!(json.is_ok());
    let deserialized: Result<LintIssue, _> =
        serde_json::from_str(&json.expect("json serialization should succeed in test"));
    assert!(deserialized.is_ok());
}
// ========== Basic Linting Tests ==========
#[test]
fn test_lint_empty_expression() {
    let linter = create_test_linter();
    let expr = create_test_expr_literal_int(42);
    let issues = linter
        .lint(&expr, "42")
        .expect("lint should succeed in test");
    assert_eq!(issues.len(), 0);
}
#[test]
fn test_lint_undefined_variable() {
    let linter = create_test_linter_with_rules("undefined");
    let expr = create_test_expr_identifier("undefined_var");
    let issues = linter
        .lint(&expr, "undefined_var")
        .expect("operation should succeed in test");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].rule, "undefined");
    assert_eq!(issues[0].name, "undefined_var");
    assert_eq!(issues[0].severity, "error");
}
#[test]
fn test_lint_builtin_functions() {
    let linter = create_test_linter_with_rules("undefined");
    let println_expr = create_test_expr_identifier("println");
    let print_expr = create_test_expr_identifier("print");
    let eprintln_expr = create_test_expr_identifier("eprintln");
    assert_eq!(
        linter
            .lint(&println_expr, "println")
            .expect("operation should succeed in test")
            .len(),
        0
    );
    assert_eq!(
        linter
            .lint(&print_expr, "print")
            .expect("operation should succeed in test")
            .len(),
        0
    );
    assert_eq!(
        linter
            .lint(&eprintln_expr, "eprintln")
            .expect("operation should succeed in test")
            .len(),
        0
    );
}
#[test]
fn test_lint_unused_variable() {
    let linter = create_test_linter_with_rules("unused");
    let expr = create_test_expr_let(
        "x",
        create_test_expr_literal_int(42),
        create_test_expr_literal_int(0),
    );
    let issues = linter
        .lint(&expr, "let x = 42; 0")
        .expect("operation should succeed in test");
    assert!(issues
        .iter()
        .any(|i| i.rule == "unused_variable" && i.name == "x"));
}
#[test]
fn test_lint_used_variable() {
    let linter = create_test_linter_with_rules("unused");
    let expr = create_test_expr_let(
        "x",
        create_test_expr_literal_int(42),
        create_test_expr_identifier("x"),
    );
    let issues = linter
        .lint(&expr, "let x = 42; x")
        .expect("operation should succeed in test");
    assert!(!issues
        .iter()
        .any(|i| i.rule == "unused_variable" && i.name == "x"));
}
#[test]
fn test_lint_variable_shadowing() {
    let linter = create_test_linter_with_rules("shadowing");
    // Direct scope test - this should trigger shadowing
    let mut parent_scope = Scope::new();
    parent_scope.define("x".to_string(), 1, 1, VarType::Local);
    let child_scope = Scope::with_parent(parent_scope);
    assert!(child_scope.is_shadowing("x"));
    // Direct test without function wrapper
    let outer_let = create_test_expr_let(
        "x",
        create_test_expr_literal_int(1),
        create_test_expr_let(
            "x", // This should shadow the outer x
            create_test_expr_literal_int(2),
            create_test_expr_identifier("x"),
        ),
    );
    let issues = linter
        .lint(&outer_let, "let x = 1; let x = 2; x")
        .expect("operation should succeed in test");
    eprintln!("Debug - Issues found: {issues:?}");
    assert!(issues
        .iter()
        .any(|i| i.rule == "shadowing" && i.name == "x"));
}
// ========== Function Linting Tests ==========
#[test]
fn test_lint_function_definition() {
    let linter = create_test_linter_with_rules("unused");
    let expr = create_test_expr_function(
        "test_func",
        vec![create_test_param("x")],
        create_test_expr_literal_int(42),
    );
    let issues = linter
        .lint(&expr, "fn test_func(x) { 42 }")
        .expect("operation should succeed in test");
    // Parameters are not flagged as unused in function scope analysis
    assert!(!issues.iter().any(|i| i.rule == "unused_parameter"));
}
#[test]
fn test_lint_function_unused_local_variable() {
    let linter = create_test_linter_with_rules("unused");
    let body = create_test_expr_let(
        "local_var",
        create_test_expr_literal_int(1),
        create_test_expr_literal_int(42),
    );
    let expr = create_test_expr_function("test_func", vec![], body);
    let issues = linter
        .lint(&expr, "fn test_func() { let local_var = 1; 42 }")
        .expect("operation should succeed in test");
    assert!(issues
        .iter()
        .any(|i| i.rule == "unused_variable" && i.name == "local_var"));
}
// ========== Loop Linting Tests ==========
#[test]
fn test_lint_for_loop_unused_variable() {
    let linter = create_test_linter_with_rules("unused");
    let expr = create_test_expr_for(
        "i",
        Some(Pattern::Identifier("i".to_string())),
        create_test_expr_literal_int(42),
        create_test_expr_literal_int(0),
    );
    let issues = linter
        .lint(&expr, "for i in items { 0 }")
        .expect("operation should succeed in test");
    assert!(issues
        .iter()
        .any(|i| i.rule.contains("unused") && i.name == "i"));
}
#[test]
fn test_lint_for_loop_used_variable() {
    let linter = create_test_linter_with_rules("unused");
    let expr = create_test_expr_for(
        "i",
        Some(Pattern::Identifier("i".to_string())),
        create_test_expr_literal_int(42),
        create_test_expr_identifier("i"),
    );
    let issues = linter
        .lint(&expr, "for i in items { i }")
        .expect("operation should succeed in test");
    assert!(!issues
        .iter()
        .any(|i| i.rule.contains("unused") && i.name == "i"));
}
#[test]
fn test_lint_for_loop_underscore_variable() {
    let linter = create_test_linter_with_rules("unused");
    let expr = create_test_expr_for(
        "_",
        Some(Pattern::Identifier("_".to_string())),
        create_test_expr_literal_int(42),
        create_test_expr_literal_int(0),
    );
    let issues = linter
        .lint(&expr, "for _ in items { 0 }")
        .expect("operation should succeed in test");
    assert!(!issues.iter().any(|i| i.name == "_"));
}
// ========== Match Expression Tests ==========
#[test]
fn test_lint_match_unused_binding() {
    let linter = create_test_linter_with_rules("unused");
    let arm = create_test_match_arm(
        Pattern::Identifier("x".to_string()),
        create_test_expr_literal_int(42),
    );
    let expr = create_test_expr_match(create_test_expr_literal_int(1), vec![arm]);
    let issues = linter
        .lint(&expr, "match value { x => 42 }")
        .expect("operation should succeed in test");
    assert!(issues
        .iter()
        .any(|i| i.rule.contains("unused") && i.name == "x"));
}
#[test]
fn test_lint_match_used_binding() {
    let linter = create_test_linter_with_rules("unused");
    let arm = create_test_match_arm(
        Pattern::Identifier("x".to_string()),
        create_test_expr_identifier("x"),
    );
    let expr = create_test_expr_match(create_test_expr_literal_int(1), vec![arm]);
    let issues = linter
        .lint(&expr, "match value { x => x }")
        .expect("operation should succeed in test");
    assert!(!issues
        .iter()
        .any(|i| i.rule.contains("unused") && i.name == "x"));
}
#[test]
fn test_lint_match_underscore_binding() {
    let linter = create_test_linter_with_rules("unused");
    let arm = create_test_match_arm(
        Pattern::Identifier("_".to_string()),
        create_test_expr_literal_int(42),
    );
    let expr = create_test_expr_match(create_test_expr_literal_int(1), vec![arm]);
    let issues = linter
        .lint(&expr, "match value { _ => 42 }")
        .expect("operation should succeed in test");
    assert!(!issues.iter().any(|i| i.name == "_"));
}
// ========== Lambda Expression Tests ==========
#[test]
fn test_lint_lambda_unused_parameter() {
    let linter = create_test_linter_with_rules("unused");
    let expr = create_test_expr_lambda(
        vec![create_test_param("x")],
        create_test_expr_literal_int(42),
    );
    let issues = linter
        .lint(&expr, "|x| 42")
        .expect("operation should succeed in test");
    assert!(issues
        .iter()
        .any(|i| i.rule.contains("unused") && i.name == "x"));
}
#[test]
fn test_lint_lambda_used_parameter() {
    let linter = create_test_linter_with_rules("unused");
    let expr = create_test_expr_lambda(
        vec![create_test_param("x")],
        create_test_expr_identifier("x"),
    );
    let issues = linter
        .lint(&expr, "|x| x")
        .expect("operation should succeed in test");
    assert!(!issues
        .iter()
        .any(|i| i.rule.contains("unused") && i.name == "x"));
}
// ========== Complexity Tests ==========
#[test]
fn test_complexity_calculation_simple() {
    let linter = create_test_linter();
    let expr = create_test_expr_literal_int(42);
    assert_eq!(Linter::calculate_complexity(&expr), 0);
}
#[test]
fn test_complexity_calculation_if() {
    let linter = create_test_linter();
    let expr = create_test_expr_if(
        create_test_expr_literal_int(1),
        create_test_expr_literal_int(2),
        Some(create_test_expr_literal_int(3)),
    );
    assert_eq!(Linter::calculate_complexity(&expr), 1);
}
#[test]
fn test_complexity_calculation_match() {
    let linter = create_test_linter();
    let arm = create_test_match_arm(
        Pattern::Identifier("_".to_string()),
        create_test_expr_literal_int(42),
    );
    let expr = create_test_expr_match(create_test_expr_literal_int(1), vec![arm]);
    assert_eq!(Linter::calculate_complexity(&expr), 2);
}
#[test]
fn test_complexity_calculation_while() {
    let linter = create_test_linter();
    let expr = create_test_expr_while(
        create_test_expr_literal_int(1),
        create_test_expr_literal_int(2),
    );
    assert_eq!(Linter::calculate_complexity(&expr), 2);
}
#[test]
fn test_complexity_calculation_for() {
    let linter = create_test_linter();
    let expr = create_test_expr_for(
        "i",
        Some(Pattern::Identifier("i".to_string())),
        create_test_expr_literal_int(42),
        create_test_expr_literal_int(0),
    );
    assert_eq!(Linter::calculate_complexity(&expr), 2);
}
#[test]
fn test_complexity_limit_violation() {
    let mut linter = create_test_linter_with_rules("complexity");
    linter.max_complexity = 1; // Very low limit
    let complex_expr = create_test_expr_if(
        create_test_expr_literal_int(1),
        create_test_expr_if(
            create_test_expr_literal_int(2),
            create_test_expr_literal_int(3),
            None,
        ),
        None,
    );
    let issues = linter
        .lint(&complex_expr, "if 1 { if 2 { 3 } }")
        .expect("operation should succeed in test");
    assert!(issues.iter().any(|i| i.rule == "complexity"));
}
#[test]
fn test_complexity_limit_strict_mode() {
    let mut linter = create_test_linter_with_rules("complexity");
    linter.set_strict_mode(true);
    linter.max_complexity = 0;
    let expr = create_test_expr_literal_int(42);
    let issues = linter
        .lint(&expr, "42")
        .expect("lint should succeed in test");
    // Simple expression should not trigger complexity
    assert!(!issues.iter().any(|i| i.rule == "complexity"));
}
// ========== Pattern Extraction Tests ==========
#[test]
fn test_extract_loop_bindings_tuple() {
    let linter = create_test_linter();
    let mut scope = Scope::new();
    let pattern = Pattern::Tuple(vec![
        Pattern::Identifier("x".to_string()),
        Pattern::Identifier("y".to_string()),
    ]);
    Linter::extract_loop_bindings(&pattern, &mut scope);
    assert!(scope.is_defined("x"));
    assert!(scope.is_defined("y"));
}
#[test]
fn test_extract_loop_bindings_list() {
    let linter = create_test_linter();
    let mut scope = Scope::new();
    let pattern = Pattern::List(vec![
        Pattern::Identifier("first".to_string()),
        Pattern::Identifier("second".to_string()),
    ]);
    Linter::extract_loop_bindings(&pattern, &mut scope);
    assert!(scope.is_defined("first"));
    assert!(scope.is_defined("second"));
}
#[test]
fn test_extract_loop_bindings_struct() {
    let linter = create_test_linter();
    let mut scope = Scope::new();
    let pattern = Pattern::Struct {
        name: "Point".to_string(),
        fields: vec![
            StructPatternField {
                name: "x".to_string(),
                pattern: Some(Pattern::Identifier("x_val".to_string())),
            },
            StructPatternField {
                name: "y".to_string(),
                pattern: None,
            },
        ],
        has_rest: false,
    };
    Linter::extract_loop_bindings(&pattern, &mut scope);
    assert!(scope.is_defined("x_val"));
    assert!(scope.is_defined("y"));
}
#[test]
fn test_extract_param_bindings_underscore() {
    let linter = create_test_linter();
    let mut scope = Scope::new();
    let pattern = Pattern::Identifier("_".to_string());
    Linter::extract_param_bindings(&pattern, &mut scope);
    assert!(!scope.is_defined("_"));
}
#[test]
fn test_extract_pattern_bindings_nested_option() {
    let linter = create_test_linter();
    let mut scope = Scope::new();
    let pattern = Pattern::Some(Box::new(Pattern::Identifier("value".to_string())));
    Linter::extract_pattern_bindings(&pattern, &mut scope);
    assert!(scope.is_defined("value"));
}
#[test]
fn test_extract_pattern_bindings_ok_err() {
    let linter = create_test_linter();
    let mut scope = Scope::new();
    let ok_pattern = Pattern::Ok(Box::new(Pattern::Identifier("success".to_string())));
    Linter::extract_pattern_bindings(&ok_pattern, &mut scope);
    assert!(scope.is_defined("success"));
    let err_pattern = Pattern::Err(Box::new(Pattern::Identifier("error".to_string())));
    Linter::extract_pattern_bindings(&err_pattern, &mut scope);
    assert!(scope.is_defined("error"));
}
// ========== Expression Analysis Tests ==========
#[test]
fn test_analyze_binary_expression() {
    let linter = create_test_linter_with_rules("undefined");
    let expr = create_test_expr_binary(
        BinaryOp::Add,
        create_test_expr_identifier("undefined_left"),
        create_test_expr_identifier("undefined_right"),
    );
    let issues = linter
        .lint(&expr, "undefined_left + undefined_right")
        .expect("operation should succeed in test");
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().any(|i| i.name == "undefined_left"));
    assert!(issues.iter().any(|i| i.name == "undefined_right"));
}
#[test]
fn test_analyze_call_expression() {
    let linter = create_test_linter_with_rules("undefined");
    let expr = create_test_expr_call(
        create_test_expr_identifier("undefined_func"),
        vec![create_test_expr_identifier("undefined_arg")],
    );
    let issues = linter
        .lint(&expr, "undefined_func(undefined_arg)")
        .expect("operation should succeed in test");
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().any(|i| i.name == "undefined_func"));
    assert!(issues.iter().any(|i| i.name == "undefined_arg"));
}
#[test]
fn test_analyze_method_call_expression() {
    let linter = create_test_linter_with_rules("undefined");
    let expr = create_test_expr_method_call(
        create_test_expr_identifier("undefined_obj"),
        "method",
        vec![create_test_expr_identifier("undefined_arg")],
    );
    let issues = linter
        .lint(&expr, "undefined_obj.method(undefined_arg)")
        .expect("operation should succeed in test");
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().any(|i| i.name == "undefined_obj"));
    assert!(issues.iter().any(|i| i.name == "undefined_arg"));
}
#[test]
fn test_analyze_string_interpolation() {
    let linter = create_test_linter_with_rules("undefined");
    let expr = Expr::new(
        ExprKind::StringInterpolation {
            parts: vec![
                StringPart::Text("Hello ".to_string()),
                StringPart::Expr(Box::new(create_test_expr_identifier("undefined_name"))),
                StringPart::ExprWithFormat {
                    expr: Box::new(create_test_expr_identifier("undefined_age")),
                    format_spec: "d".to_string(),
                },
            ],
        },
        create_test_span(),
    );
    let issues = linter
        .lint(&expr, "f\"Hello {undefined_name} {undefined_age:d}\"")
        .expect("operation should succeed in test");
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().any(|i| i.name == "undefined_name"));
    assert!(issues.iter().any(|i| i.name == "undefined_age"));
}
#[test]
fn test_analyze_return_expression() {
    let linter = create_test_linter_with_rules("undefined");
    let expr = create_test_expr_return(Some(create_test_expr_identifier("undefined_var")));
    let issues = linter
        .lint(&expr, "return undefined_var")
        .expect("operation should succeed in test");
    assert_eq!(issues.len(), 1);
    assert!(issues.iter().any(|i| i.name == "undefined_var"));
}
#[test]
fn test_analyze_list_and_tuple() {
    let linter = create_test_linter_with_rules("undefined");
    let list_expr = Expr::new(
        ExprKind::List(vec![create_test_expr_identifier("undefined_item")]),
        create_test_span(),
    );
    let tuple_expr = Expr::new(
        ExprKind::Tuple(vec![create_test_expr_identifier("undefined_elem")]),
        create_test_span(),
    );
    let list_issues = linter
        .lint(&list_expr, "[undefined_item]")
        .expect("operation should succeed in test");
    assert!(list_issues.iter().any(|i| i.name == "undefined_item"));
    let tuple_issues = linter
        .lint(&tuple_expr, "(undefined_elem,)")
        .expect("operation should succeed in test");
    assert!(tuple_issues.iter().any(|i| i.name == "undefined_elem"));
}
#[test]
fn test_analyze_field_and_index_access() {
    let linter = create_test_linter_with_rules("undefined");
    let field_expr = Expr::new(
        ExprKind::FieldAccess {
            object: Box::new(create_test_expr_identifier("undefined_obj")),
            field: "property".to_string(),
        },
        create_test_span(),
    );
    let index_expr = Expr::new(
        ExprKind::IndexAccess {
            object: Box::new(create_test_expr_identifier("undefined_arr")),
            index: Box::new(create_test_expr_identifier("undefined_idx")),
        },
        create_test_span(),
    );
    let field_issues = linter
        .lint(&field_expr, "undefined_obj.property")
        .expect("operation should succeed in test");
    assert!(field_issues.iter().any(|i| i.name == "undefined_obj"));
    let index_issues = linter
        .lint(&index_expr, "undefined_arr[undefined_idx]")
        .expect("operation should succeed in test");
    assert_eq!(index_issues.len(), 2);
    assert!(index_issues.iter().any(|i| i.name == "undefined_arr"));
    assert!(index_issues.iter().any(|i| i.name == "undefined_idx"));
}
#[test]
fn test_analyze_assign_expression() {
    let linter = create_test_linter_with_rules("undefined");
    let expr = Expr::new(
        ExprKind::Assign {
            target: Box::new(create_test_expr_identifier("undefined_target")),
            value: Box::new(create_test_expr_identifier("undefined_value")),
        },
        create_test_span(),
    );
    let issues = linter
        .lint(&expr, "undefined_target = undefined_value")
        .expect("operation should succeed in test");
    assert_eq!(issues.len(), 2);
    assert!(issues.iter().any(|i| i.name == "undefined_target"));
    assert!(issues.iter().any(|i| i.name == "undefined_value"));
}
// ========== Block Scope Tests ==========
#[test]
fn test_analyze_block_unused_variable() {
    let linter = create_test_linter_with_rules("unused");
    let block = create_test_expr_block(vec![create_test_expr_let(
        "unused_var",
        create_test_expr_literal_int(42),
        create_test_expr_literal_int(0),
    )]);
    let issues = linter
        .lint(&block, "{ let unused_var = 42; 0 }")
        .expect("operation should succeed in test");
    assert!(issues
        .iter()
        .any(|i| i.rule == "unused_variable" && i.name == "unused_var"));
}
#[test]
fn test_analyze_if_branches() {
    let linter = create_test_linter_with_rules("undefined");
    let expr = create_test_expr_if(
        create_test_expr_identifier("undefined_cond"),
        create_test_expr_identifier("undefined_then"),
        Some(create_test_expr_identifier("undefined_else")),
    );
    let issues = linter
        .lint(
            &expr,
            "if undefined_cond { undefined_then } else { undefined_else }",
        )
        .expect("operation should succeed in test");
    assert_eq!(issues.len(), 3);
    assert!(issues.iter().any(|i| i.name == "undefined_cond"));
    assert!(issues.iter().any(|i| i.name == "undefined_then"));
    assert!(issues.iter().any(|i| i.name == "undefined_else"));
}
// ========== Auto-fix Tests ==========
#[test]
fn test_auto_fix_style_issue() {
    let linter = create_test_linter();
    let issues = vec![LintIssue {
        line: 1,
        column: 1,
        severity: "warning".to_string(),
        rule: "style".to_string(),
        message: "double spaces".to_string(),
        suggestion: "Use single spaces".to_string(),
        issue_type: "style".to_string(),
        name: "spacing".to_string(),
    }];
    let fixed = linter
        .auto_fix("let  x  =  42", &issues)
        .expect("operation should succeed in test");
    assert_eq!(fixed, "let x = 42");
}
#[test]
fn test_auto_fix_no_issues() {
    let linter = create_test_linter();
    let issues = vec![];
    let fixed = linter
        .auto_fix("let x = 42", &issues)
        .expect("operation should succeed in test");
    assert_eq!(fixed, "let x = 42");
}
// ========== Integration Tests ==========
#[test]
fn test_comprehensive_linting() {
    let linter = create_test_linter_with_rules("unused,undefined,shadowing");
    // Create nested let expressions for comprehensive testing
    let unused_let = create_test_expr_let(
        "unused",
        create_test_expr_identifier("undefined"), // This creates undefined variable
        create_test_expr_identifier("x"),
    );
    let shadow_let = create_test_expr_let(
        "x", // This shadows the outer x
        create_test_expr_literal_int(2),
        unused_let,
    );
    let outer_let = create_test_expr_let(
        "x", // Outer variable
        create_test_expr_literal_int(1),
        shadow_let,
    );
    let issues = linter
        .lint(&outer_let, "complex code")
        .expect("operation should succeed in test");
    assert!(issues.iter().any(|i| i.rule == "shadowing"));
    assert!(issues.iter().any(|i| i.rule == "undefined"));
    assert!(issues.iter().any(|i| i.rule == "unused_variable"));
}
#[test]
fn test_variable_type_classification() {
    let var_info = VariableInfo {
        defined_at: (1, 1),
        used: false,
        var_type: VarType::Parameter,
    };
    assert_eq!(var_info.defined_at, (1, 1));
    assert!(!var_info.used);
    assert!(matches!(var_info.var_type, VarType::Parameter));
}
#[test]
fn test_empty_issues_json_compatibility() {
    let linter = create_test_linter();
    let expr = create_test_expr_literal_int(42);
    let issues = linter
        .lint(&expr, "42")
        .expect("lint should succeed in test");
    assert_eq!(issues.len(), 0);
    let json = serde_json::to_string(&issues).expect("operation should succeed in test");
    assert_eq!(json, "[]");
}

#[test]
fn test_lint_rules_enum_coverage() {
    // Test all LintRule variants can be created
    let _unused_var = LintRule::UnusedVariable;
    let _undefined_var = LintRule::UndefinedVariable;
    let _variable_shadowing = LintRule::VariableShadowing;
    let _unused_param = LintRule::UnusedParameter;
    let _unused_loop_var = LintRule::UnusedLoopVariable;
    let _unused_match_binding = LintRule::UnusedMatchBinding;
    let _complexity_limit = LintRule::ComplexityLimit;
    let _naming_convention = LintRule::NamingConvention;
    let _style_violation = LintRule::StyleViolation;
    let _security = LintRule::Security;
    let _performance = LintRule::Performance;
    // Test passes without panic; // All variants created successfully
}

#[test]
fn test_var_type_enum_coverage() {
    // Test all VarType variants can be created
    let _local = VarType::Local;
    let _parameter = VarType::Parameter;
    let _loop_variable = VarType::LoopVariable;
    let _match_binding = VarType::MatchBinding;
    // Test passes without panic; // All variants created successfully
}

#[test]
fn test_variable_info_structure() {
    let var_info = VariableInfo {
        defined_at: (1, 5),
        used: false,
        var_type: VarType::Local,
    };
    assert_eq!(var_info.defined_at, (1, 5));
    assert!(!var_info.used);
    assert!(matches!(var_info.var_type, VarType::Local));
}

#[test]
fn test_scope_creation_duplicate_renamed() {
    let scope = Scope {
        variables: HashMap::new(),
        parent: None,
    };
    assert_eq!(scope.variables.len(), 0);
    assert!(scope.parent.is_none());
}

#[test]
fn test_lint_issue_serialization_duplicate_renamed() {
    let issue = LintIssue {
        line: 10,
        column: 5,
        severity: "warning".to_string(),
        rule: "unused-variable".to_string(),
        message: "Variable 'x' is never used".to_string(),
        suggestion: "Remove unused variable".to_string(),
        issue_type: "unused".to_string(),
        name: "x".to_string(),
    };

    let json = serde_json::to_string(&issue).expect("operation should succeed in test");
    assert!(json.contains("\"line\":10"));
    assert!(json.contains("\"column\":5"));
    assert!(json.contains("\"severity\":\"warning\""));
    assert!(json.contains("\"rule\":\"unused-variable\""));
    assert!(json.contains("\"type\":\"unused\""));

    // Test deserialization
    let deserialized: LintIssue =
        serde_json::from_str(&json).expect("operation should succeed in test");
    assert_eq!(deserialized.line, 10);
    assert_eq!(deserialized.column, 5);
    assert_eq!(deserialized.severity, "warning");
    assert_eq!(deserialized.rule, "unused-variable");
}

#[test]
fn test_linter_with_all_rules() {
    let mut linter = Linter::new();
    linter.rules.clear(); // Clear default rules first
    linter.add_rule(LintRule::UnusedVariable);
    linter.add_rule(LintRule::UndefinedVariable);
    linter.add_rule(LintRule::VariableShadowing);
    linter.add_rule(LintRule::UnusedParameter);
    linter.add_rule(LintRule::UnusedLoopVariable);
    linter.add_rule(LintRule::UnusedMatchBinding);
    linter.add_rule(LintRule::ComplexityLimit);
    linter.add_rule(LintRule::NamingConvention);
    linter.add_rule(LintRule::StyleViolation);
    linter.add_rule(LintRule::Security);
    linter.add_rule(LintRule::Performance);

    assert_eq!(linter.rules.len(), 11);
}

#[test]
fn test_lint_issue_debug_format() {
    let issue = LintIssue {
        line: 1,
        column: 1,
        severity: "error".to_string(),
        rule: "test-rule".to_string(),
        message: "Test message".to_string(),
        suggestion: "Test suggestion".to_string(),
        issue_type: "test".to_string(),
        name: "test_name".to_string(),
    };

    let debug_str = format!("{issue:?}");
    assert!(debug_str.contains("LintIssue"));
    assert!(debug_str.contains("line: 1"));
    assert!(debug_str.contains("error"));
    assert!(debug_str.contains("test-rule"));
}

#[test]
fn test_variable_info_debug_format() {
    let var_info = VariableInfo {
        defined_at: (5, 10),
        used: true,
        var_type: VarType::Parameter,
    };

    let debug_str = format!("{var_info:?}");
    assert!(debug_str.contains("VariableInfo"));
    assert!(debug_str.contains("defined_at"));
    assert!(debug_str.contains("used: true"));
    assert!(debug_str.contains("Parameter"));
}

#[test]
fn test_lint_rules_debug_format() {
    let rules = [
        LintRule::UnusedVariable,
        LintRule::UndefinedVariable,
        LintRule::ComplexityLimit,
    ];

    for rule in rules {
        let debug_str = format!("{rule:?}");
        assert!(!debug_str.is_empty());
    }
}

impl Linter {
    pub fn add_rule(&mut self, rule: LintRule) {
        self.rules.push(rule);
    }
}
