/// Custom lint rules for Ruchy code quality
use crate::frontend::ast::{Expr, ExprKind, Module, Statement};
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
    fn check_module(&self, module: &Module) -> Vec<LintViolation>;
    fn check_statement(&self, _stmt: &Statement) -> Vec<LintViolation> {
        vec![]
    }
    fn check_expression(&self, _expr: &Expr) -> Vec<LintViolation> {
        vec![]
    }
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
            Box::new(NoUnwrapRule),
            Box::new(ComplexityRule::default()),
            Box::new(NamingConventionRule),
            Box::new(NoTodoCommentsRule),
            Box::new(FunctionLengthRule::default()),
            Box::new(NoDebugPrintRule),
        ];
        
        Self { rules }
    }
    
    pub fn add_rule(&mut self, rule: Box<dyn LintRule>) {
        self.rules.push(rule);
    }
    
    pub fn lint(&self, module: &Module) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        
        for rule in &self.rules {
            violations.extend(rule.check_module(module));
        }
        
        violations
    }
}

/// Rule: No unwrap() calls in production code
struct NoUnwrapRule;

impl LintRule for NoUnwrapRule {
    fn name(&self) -> &str {
        "no_unwrap"
    }
    
    fn check_module(&self, module: &Module) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        
        // Walk the AST looking for unwrap calls
        for stmt in &module.statements {
            violations.extend(self.check_statement(stmt));
        }
        
        violations
    }
    
    fn check_expression(&self, expr: &Expr) -> Vec<LintViolation> {
        match &expr.kind {
            ExprKind::MethodCall { object, method, .. } => {
                let mut violations = Vec::new();
                
                if method == "unwrap" {
                    violations.push(LintViolation::Violation {
                        location: format!("line {}", expr.span.start.line),
                        message: "Use of unwrap() is discouraged".to_string(),
                        severity: Severity::Error,
                        suggestion: Some("Use ? operator or expect() with a descriptive message".to_string()),
                    });
                }
                
                // Recursively check the object
                violations.extend(self.check_expression(object));
                violations
            }
            _ => vec![]
        }
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
            ExprKind::While { condition, body } | ExprKind::For { condition, body, .. } => {
                1 + self.calculate_complexity(condition) + self.calculate_complexity(body)
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
    
    fn check_module(&self, module: &Module) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let max = if self.max_complexity == 0 { 10 } else { self.max_complexity };
        
        for stmt in &module.statements {
            if let Statement::Function { name, body, .. } = stmt {
                let complexity = self.calculate_complexity(body);
                if complexity > max {
                    violations.push(LintViolation::Violation {
                        location: format!("function {}", name),
                        message: format!("Cyclomatic complexity is {} (max: {})", complexity, max),
                        severity: Severity::Warning,
                        suggestion: Some("Consider breaking this function into smaller functions".to_string()),
                    });
                }
            }
        }
        
        violations
    }
}

/// Rule: Naming conventions
struct NamingConventionRule;

impl LintRule for NamingConventionRule {
    fn name(&self) -> &str {
        "naming_convention"
    }
    
    fn check_module(&self, module: &Module) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        
        for stmt in &module.statements {
            match stmt {
                Statement::Function { name, .. } => {
                    if !name.chars().next().map_or(false, |c| c.is_lowercase()) {
                        violations.push(LintViolation::Violation {
                            location: format!("function {}", name),
                            message: "Function names should be snake_case".to_string(),
                            severity: Severity::Warning,
                            suggestion: Some(format!("Rename to {}", to_snake_case(name))),
                        });
                    }
                }
                Statement::Struct { name, .. } => {
                    if !name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        violations.push(LintViolation::Violation {
                            location: format!("struct {}", name),
                            message: "Struct names should be PascalCase".to_string(),
                            severity: Severity::Warning,
                            suggestion: Some(format!("Rename to {}", to_pascal_case(name))),
                        });
                    }
                }
                Statement::Let { name, .. } => {
                    if !name.chars().next().map_or(false, |c| c.is_lowercase()) {
                        violations.push(LintViolation::Violation {
                            location: format!("variable {}", name),
                            message: "Variable names should be snake_case".to_string(),
                            severity: Severity::Warning,
                            suggestion: Some(format!("Rename to {}", to_snake_case(name))),
                        });
                    }
                }
                _ => {}
            }
        }
        
        violations
    }
}

/// Rule: No TODO/FIXME comments
struct NoTodoCommentsRule;

impl LintRule for NoTodoCommentsRule {
    fn name(&self) -> &str {
        "no_todo_comments"
    }
    
    fn check_module(&self, _module: &Module) -> Vec<LintViolation> {
        // This would need access to the original source with comments
        // For now, return empty
        vec![]
    }
}

/// Rule: Function length limit
#[derive(Default)]
struct FunctionLengthRule {
    max_lines: usize,
}

impl LintRule for FunctionLengthRule {
    fn name(&self) -> &str {
        "function_length"
    }
    
    fn check_module(&self, module: &Module) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let max = if self.max_lines == 0 { 50 } else { self.max_lines };
        
        for stmt in &module.statements {
            if let Statement::Function { name, body, .. } = stmt {
                // Estimate lines based on AST depth (simplified)
                let estimated_lines = estimate_lines(body);
                if estimated_lines > max {
                    violations.push(LintViolation::Violation {
                        location: format!("function {}", name),
                        message: format!("Function is approximately {} lines (max: {})", estimated_lines, max),
                        severity: Severity::Warning,
                        suggestion: Some("Consider breaking this function into smaller functions".to_string()),
                    });
                }
            }
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
    
    fn check_module(&self, module: &Module) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        
        for stmt in &module.statements {
            violations.extend(self.check_statement(stmt));
        }
        
        violations
    }
    
    fn check_expression(&self, expr: &Expr) -> Vec<LintViolation> {
        match &expr.kind {
            ExprKind::FunctionCall { name, .. } => {
                if name == "dbg" || name == "debug_print" {
                    vec![LintViolation::Violation {
                        location: format!("line {}", expr.span.start.line),
                        message: "Debug print statement found".to_string(),
                        severity: Severity::Warning,
                        suggestion: Some("Remove debug statements before committing".to_string()),
                    }]
                } else {
                    vec![]
                }
            }
            _ => vec![]
        }
    }
}

// Helper functions
fn to_snake_case(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if i > 0 && c.is_uppercase() {
                format!("_{}", c.to_lowercase())
            } else {
                c.to_lowercase().to_string()
            }
        })
        .collect()
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn estimate_lines(expr: &Expr) -> usize {
    // Simplified line estimation based on AST structure
    match &expr.kind {
        ExprKind::Block { statements } => {
            1 + statements.iter().map(|s| estimate_statement_lines(s)).sum::<usize>()
        }
        ExprKind::If { then_branch, else_branch, .. } => {
            2 + estimate_lines(then_branch) 
                + else_branch.as_ref().map_or(0, |e| 1 + estimate_lines(e))
        }
        ExprKind::Match { arms, .. } => {
            2 + arms.len() + arms.iter().map(|arm| estimate_lines(&arm.body)).sum::<usize>()
        }
        _ => 1
    }
}

fn estimate_statement_lines(stmt: &Statement) -> usize {
    match stmt {
        Statement::Expression(expr) => estimate_lines(expr),
        Statement::Let { value, .. } => 1 + value.as_ref().map_or(0, estimate_lines),
        _ => 1
    }
}