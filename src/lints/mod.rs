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
    fn calculate_complexity(&self, expr: &Expr) -> usize {
        match &expr.kind {
            ExprKind::If { condition, then_branch, else_branch } => {
                1 + self.calculate_complexity(condition)
                    + self.calculate_complexity(then_branch)
                    + else_branch.as_ref().map_or(0, |e| self.calculate_complexity(e))
            }
            ExprKind::Match { expr, arms } => {
                1 + self.calculate_complexity(expr)
                    + arms.iter().map(|arm| self.calculate_complexity(&arm.body)).sum::<usize>()
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
            _ => 0
        }
    }
}

impl LintRule for ComplexityRule {
    fn name(&self) -> &str {
        "complexity"
    }
    
    fn check_expression(&self, expr: &Expr) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let max = if self.max_complexity == 0 { 10 } else { self.max_complexity };
        
        let complexity = self.calculate_complexity(expr);
        if complexity > max {
            violations.push(LintViolation::Violation {
                location: format!("position {}", expr.span.start),
                message: format!("Cyclomatic complexity is {} (max: {})", complexity, max),
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
    fn name(&self) -> &str {
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
                            suggestion: Some("Remove debug statements before committing".to_string()),
                        }]
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            }
            _ => vec![]
        }
    }
}