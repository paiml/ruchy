/// Custom lint rules for Ruchy code quality
use crate::frontend::ast::{Expr, ExprKind};
use thiserror::Error;
#[derive(Debug, Error)]
pub enum LintViolation {
    #[error("{location}: {message} (severity: {severity:?})")]
    Violation {
        location: String,
        message: String,
        severity: Severity,
        suggestion: Option<String>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}
/// Trait for implementing lint rules
pub trait LintRule: Send + Sync {
    fn name(&self) -> &str;
    fn check_expression(&self, expr: &Expr) -> Vec<LintViolation>;
}
/// Main linter that runs all rules
pub struct RuchyLinter {
    rules: Vec<Box<dyn LintRule>>,
}
impl Default for RuchyLinter {
    fn default() -> Self {
        Self::new()
    }
}
impl RuchyLinter {
    pub fn new() -> Self {
        let rules: Vec<Box<dyn LintRule>> = vec![
            Box::new(ComplexityRule::default()),
            Box::new(NoDebugPrintRule),
        ];
        Self { rules }
    }
    pub fn add_rule(&mut self, rule: Box<dyn LintRule>) {
        self.rules.push(rule);
    }
    pub fn lint(&self, expr: &Expr) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        for rule in &self.rules {
            violations.extend(rule.check_expression(expr));
        }
        violations
    }
}
/// Rule: Cyclomatic complexity limit
#[derive(Default)]
struct ComplexityRule {
    max_complexity: usize,
}
impl ComplexityRule {
    #[allow(clippy::only_used_in_recursion)]
    fn calculate_complexity(&self, expr: &Expr) -> usize {
        match &expr.kind {
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                1 + self.calculate_complexity(condition)
                    + self.calculate_complexity(then_branch)
                    + else_branch
                        .as_ref()
                        .map_or(0, |e| self.calculate_complexity(e))
            }
            ExprKind::Match { expr, arms } => {
                1 + self.calculate_complexity(expr)
                    + arms
                        .iter()
                        .map(|arm| self.calculate_complexity(&arm.body))
                        .sum::<usize>()
            }
            ExprKind::While { condition, body } => {
                1 + self.calculate_complexity(condition) + self.calculate_complexity(body)
            }
            ExprKind::For { iter, body, .. } => {
                1 + self.calculate_complexity(iter) + self.calculate_complexity(body)
            }
            ExprKind::Binary { left, right, .. } => {
                self.calculate_complexity(left) + self.calculate_complexity(right)
            }
            _ => 0,
        }
    }
}
impl LintRule for ComplexityRule {
    fn name(&self) -> &'static str {
        "complexity"
    }
    fn check_expression(&self, expr: &Expr) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let max = if self.max_complexity == 0 {
            10
        } else {
            self.max_complexity
        };
        let complexity = self.calculate_complexity(expr);
        if complexity > max {
            violations.push(LintViolation::Violation {
                location: format!("position {}", expr.span.start),
                message: format!("Cyclomatic complexity is {complexity} (max: {max})"),
                severity: Severity::Warning,
                suggestion: Some("Consider breaking this into smaller functions".to_string()),
            });
        }
        violations
    }
}
/// Rule: No debug print statements
struct NoDebugPrintRule;
impl LintRule for NoDebugPrintRule {
    fn name(&self) -> &'static str {
        "no_debug_print"
    }
    fn check_expression(&self, expr: &Expr) -> Vec<LintViolation> {
        match &expr.kind {
            ExprKind::Call { func, .. } => {
                if let ExprKind::Identifier(name) = &func.kind {
                    if name == "dbg" || name == "debug_print" {
                        vec![LintViolation::Violation {
                            location: format!("position {}", expr.span.start),
                            message: "Debug print statement found".to_string(),
                            severity: Severity::Warning,
                            suggestion: Some(
                                "Remove debug statements before committing".to_string(),
                            ),
                        }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{Expr, ExprKind, Span, Literal, BinaryOp};
    fn make_test_expr(kind: ExprKind) -> Expr {
        Expr {
            kind,
            span: Span::new(0, 10),
            attributes: vec![],
        }
    }
    #[test]
    fn test_linter_creation() {
        let linter = RuchyLinter::new();
        assert_eq!(linter.rules.len(), 2);
    }
    #[test]
    fn test_linter_default() {
        let linter = RuchyLinter::default();
        assert_eq!(linter.rules.len(), 2);
    }
    #[test]
    fn test_add_custom_rule() {
        struct TestRule;
        impl LintRule for TestRule {
            fn name(&self) -> &'static str {
                "test"
            }
            fn check_expression(&self, _expr: &Expr) -> Vec<LintViolation> {
                vec![]
            }
        }
        let mut linter = RuchyLinter::new();
        linter.add_rule(Box::new(TestRule));
        assert_eq!(linter.rules.len(), 3);
    }
    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{:?}", Severity::Error), "Error");
        assert_eq!(format!("{:?}", Severity::Warning), "Warning");
        assert_eq!(format!("{:?}", Severity::Info), "Info");
    }
    #[test]
    fn test_severity_equality() {
        assert_eq!(Severity::Error, Severity::Error);
        assert_ne!(Severity::Error, Severity::Warning);
    }
    #[test]
    fn test_lint_violation_display() {
        let violation = LintViolation::Violation {
            location: "line 5".to_string(),
            message: "Test violation".to_string(),
            severity: Severity::Warning,
            suggestion: Some("Fix it".to_string()),
        };
        let display = violation.to_string();
        assert!(display.contains("line 5"));
        assert!(display.contains("Test violation"));
        assert!(display.contains("Warning"));
    }
    #[test]
    fn test_lint_violation_without_suggestion() {
        let violation = LintViolation::Violation {
            location: "position 10".to_string(),
            message: "Error found".to_string(),
            severity: Severity::Error,
            suggestion: None,
        };
        let display = violation.to_string();
        assert!(display.contains("position 10"));
        assert!(display.contains("Error found"));
        assert!(display.contains("Error"));
    }
    #[test]
    fn test_complexity_rule_simple() {
        let rule = ComplexityRule::default();
        let expr = make_test_expr(ExprKind::Literal(Literal::Integer(42)));
        let violations = rule.check_expression(&expr);
        assert!(violations.is_empty());
    }
    #[test]
    fn test_complexity_rule_name() {
        let rule = ComplexityRule::default();
        assert_eq!(rule.name(), "complexity");
    }
    #[test]
    fn test_complexity_rule_if_statement() {
        let rule = ComplexityRule { max_complexity: 0 }; // Will use default of 10
        // Create a simple if statement
        let if_expr = make_test_expr(ExprKind::If {
            condition: Box::new(make_test_expr(ExprKind::Literal(Literal::Bool(true)))),
            then_branch: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(1)))),
            else_branch: None,
        });
        let violations = rule.check_expression(&if_expr);
        assert!(violations.is_empty()); // Complexity is 1, under limit of 10
    }
    #[test]
    fn test_complexity_rule_nested_if_exceeds_limit() {
        let rule = ComplexityRule { max_complexity: 1 };
        // Create nested if statements to exceed complexity of 1
        let inner_if = make_test_expr(ExprKind::If {
            condition: Box::new(make_test_expr(ExprKind::Literal(Literal::Bool(true)))),
            then_branch: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(1)))),
            else_branch: None,
        });
        let outer_if = make_test_expr(ExprKind::If {
            condition: Box::new(make_test_expr(ExprKind::Literal(Literal::Bool(false)))),
            then_branch: Box::new(inner_if),
            else_branch: Some(Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(2))))),
        });
        let violations = rule.check_expression(&outer_if);
        assert!(!violations.is_empty());
        assert!(violations[0].to_string().contains("Cyclomatic complexity"));
    }
    #[test]
    fn test_complexity_rule_while_loop() {
        let rule = ComplexityRule { max_complexity: 5 };
        let while_expr = make_test_expr(ExprKind::While {
            condition: Box::new(make_test_expr(ExprKind::Literal(Literal::Bool(true)))),
            body: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(42)))),
        });
        let violations = rule.check_expression(&while_expr);
        assert!(violations.is_empty()); // Complexity is 1, under limit
    }
    #[test]
    fn test_complexity_rule_for_loop() {
        let rule = ComplexityRule { max_complexity: 5 };
        let for_expr = make_test_expr(ExprKind::For {
            var: "i".to_string(),
            pattern: None,
            iter: Box::new(make_test_expr(ExprKind::Range {
                start: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(0)))),
                end: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(10)))),
                inclusive: false,
            })),
            body: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(42)))),
        });
        let violations = rule.check_expression(&for_expr);
        assert!(violations.is_empty()); // Complexity is 1, under limit
    }
    #[test]
    fn test_complexity_rule_binary_operation() {
        let rule = ComplexityRule { max_complexity: 5 };
        let binary_expr = make_test_expr(ExprKind::Binary {
            left: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(1)))),
            op: BinaryOp::Add,
            right: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(2)))),
        });
        let violations = rule.check_expression(&binary_expr);
        assert!(violations.is_empty()); // Binary operations don't add complexity
    }
    #[test]
    fn test_no_debug_print_rule() {
        let rule = NoDebugPrintRule;
        // Test normal function call - no violation
        let normal_call = make_test_expr(ExprKind::Call {
            func: Box::new(make_test_expr(ExprKind::Identifier("println".to_string()))),
            args: vec![],
        });
        assert!(rule.check_expression(&normal_call).is_empty());
        // Test debug print - should have violation
        let debug_call = make_test_expr(ExprKind::Call {
            func: Box::new(make_test_expr(ExprKind::Identifier("dbg".to_string()))),
            args: vec![],
        });
        let violations = rule.check_expression(&debug_call);
        assert_eq!(violations.len(), 1);
        assert!(violations[0].to_string().contains("Debug print"));
        // Test debug_print - should have violation
        let debug_print = make_test_expr(ExprKind::Call {
            func: Box::new(make_test_expr(ExprKind::Identifier("debug_print".to_string()))),
            args: vec![],
        });
        let violations = rule.check_expression(&debug_print);
        assert_eq!(violations.len(), 1);
    }
    #[test]
    fn test_no_debug_print_rule_name() {
        let rule = NoDebugPrintRule;
        assert_eq!(rule.name(), "no_debug_print");
    }
    #[test]
    fn test_linter_runs_all_rules() {
        let linter = RuchyLinter::new();
        // Create expression that violates debug print rule
        let expr = make_test_expr(ExprKind::Call {
            func: Box::new(make_test_expr(ExprKind::Identifier("dbg".to_string()))),
            args: vec![],
        });
        let violations = linter.lint(&expr);
        assert!(!violations.is_empty());
    }
    #[test]
    fn test_linter_no_violations() {
        let linter = RuchyLinter::new();
        // Create simple expression with no violations
        let expr = make_test_expr(ExprKind::Literal(Literal::Integer(42)));
        let violations = linter.lint(&expr);
        assert!(violations.is_empty());
    }
    #[test]
    fn test_complexity_calculation_match() {
        let rule = ComplexityRule { max_complexity: 10 };
        // Match expressions add complexity
        use crate::frontend::ast::{MatchArm, Pattern};
        let arms = vec![
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(1)),
                guard: None,
                body: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(10)))),
                span: Span::new(0, 10),
            },
            MatchArm {
                pattern: Pattern::Literal(Literal::Integer(2)),
                guard: None,
                body: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(20)))),
                span: Span::new(0, 10),
            },
        ];
        let match_expr = make_test_expr(ExprKind::Match {
            expr: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(42)))),
            arms,
        });
        let violations = rule.check_expression(&match_expr);
        assert!(violations.is_empty()); // Under complexity limit
    }
    #[test]
    fn test_complexity_rule_with_custom_max() {
        let rule = ComplexityRule { max_complexity: 3 };
        // Simple literal should not trigger
        let expr = make_test_expr(ExprKind::Literal(Literal::Integer(42)));
        let violations = rule.check_expression(&expr);
        assert!(violations.is_empty());
    }
    #[test]
    fn test_no_debug_print_rule_non_call_expression() {
        let rule = NoDebugPrintRule;
        // Test with non-call expression
        let expr = make_test_expr(ExprKind::Literal(Literal::String("test".to_string())));
        let violations = rule.check_expression(&expr);
        assert!(violations.is_empty());
    }
    #[test]
    fn test_no_debug_print_rule_non_identifier_function() {
        let rule = NoDebugPrintRule;
        // Test with non-identifier function (e.g., lambda call)
        let expr = make_test_expr(ExprKind::Call {
            func: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(42)))),
            args: vec![],
        });
        let violations = rule.check_expression(&expr);
        assert!(violations.is_empty());
    }
    #[test]
    fn test_lint_violation_debug_formatting() {
        let violation = LintViolation::Violation {
            location: "test.ruchy:10:5".to_string(),
            message: "Complex expression detected".to_string(),
            severity: Severity::Info,
            suggestion: None,
        };
        let debug_str = format!("{violation:?}");
        assert!(debug_str.contains("Violation"));
        assert!(debug_str.contains("test.ruchy:10:5"));
    }
    #[test]
    fn test_severity_clone() {
        let sev1 = Severity::Warning;
        let sev2 = sev1;
        assert_eq!(sev1, sev2);
    }
    #[test]
    fn test_complexity_rule_if_with_else() {
        let rule = ComplexityRule { max_complexity: 5 };
        let if_expr = make_test_expr(ExprKind::If {
            condition: Box::new(make_test_expr(ExprKind::Literal(Literal::Bool(true)))),
            then_branch: Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(1)))),
            else_branch: Some(Box::new(make_test_expr(ExprKind::Literal(Literal::Integer(2))))),
        });
        let violations = rule.check_expression(&if_expr);
        assert!(violations.is_empty());
    }
    #[test]
    fn test_multiple_violations_from_linter() {
        let mut linter = RuchyLinter::new();
        // Add another rule that always fails
        struct AlwaysFailRule;
        impl LintRule for AlwaysFailRule {
            fn name(&self) -> &'static str {
                "always_fail"
            }
            fn check_expression(&self, expr: &Expr) -> Vec<LintViolation> {
                vec![LintViolation::Violation {
                    location: format!("position {}", expr.span.start),
                    message: "Always fails".to_string(),
                    severity: Severity::Error,
                    suggestion: None,
                }]
            }
        }
        linter.add_rule(Box::new(AlwaysFailRule));
        // Create expression that also violates debug print rule
        let expr = make_test_expr(ExprKind::Call {
            func: Box::new(make_test_expr(ExprKind::Identifier("dbg".to_string()))),
            args: vec![],
        });
        let violations = linter.lint(&expr);
        assert!(violations.len() >= 2); // At least 2 violations
    }
}
