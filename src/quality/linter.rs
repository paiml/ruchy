// Code linter for Ruchy
// Toyota Way: Catch issues early through static analysis

use anyhow::Result;
use crate::frontend::ast::{Expr, ExprKind};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub line: usize,
    pub column: usize,
    pub severity: String,
    pub rule: String,
    pub message: String,
    pub suggestion: String,
}

#[derive(Debug, Clone)]
pub enum LintRule {
    UnusedVariable,
    ComplexityLimit,
    NamingConvention,
    StyleViolation,
    Security,
    Performance,
}

pub struct Linter {
    rules: Vec<LintRule>,
    strict_mode: bool,
    max_complexity: usize,
}

impl Linter {
    pub fn new() -> Self {
        Self {
            rules: vec![
                LintRule::UnusedVariable,
                LintRule::ComplexityLimit,
                LintRule::NamingConvention,
            ],
            strict_mode: false,
            max_complexity: 10,
        }
    }
    
    pub fn set_rules(&mut self, rule_filter: &str) {
        self.rules.clear();
        for rule in rule_filter.split(',') {
            match rule.trim() {
                "unused" => self.rules.push(LintRule::UnusedVariable),
                "complexity" => self.rules.push(LintRule::ComplexityLimit),
                "style" => self.rules.push(LintRule::StyleViolation),
                "security" => self.rules.push(LintRule::Security),
                "performance" => self.rules.push(LintRule::Performance),
                _ => {}
            }
        }
    }
    
    pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }
    
    pub fn lint(&self, ast: &Expr, _source: &str) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();
        
        // Check for unused variables in let expressions
        self.check_unused_variables(ast, &mut issues);
        
        // Check complexity
        if self.calculate_complexity(ast) > self.max_complexity {
            issues.push(LintIssue {
                line: 1,
                column: 1,
                severity: if self.strict_mode { "error" } else { "warning" }.to_string(),
                rule: "complexity".to_string(),
                message: format!("Function complexity exceeds limit of {}", self.max_complexity),
                suggestion: "Consider breaking this into smaller functions".to_string(),
            });
        }
        
        Ok(issues)
    }
    
    pub fn auto_fix(&self, source: &str, issues: &[LintIssue]) -> Result<String> {
        // Simple auto-fix implementation
        let mut fixed = source.to_string();
        
        for issue in issues {
            if issue.rule == "style" {
                // Fix style issues
                fixed = fixed.replace("  ", " ");
            }
        }
        
        Ok(fixed)
    }
    
    fn check_unused_variables(&self, expr: &Expr, issues: &mut Vec<LintIssue>) {
        match &expr.kind {
            ExprKind::Let { name, value: _, body, .. } => {
                // Check if variable is used in body
                if !self.is_variable_used(name, body) {
                    issues.push(LintIssue {
                        line: 1,
                        column: 1,
                        severity: "warning".to_string(),
                        rule: "unused-variable".to_string(),
                        message: format!("Variable '{name}' is defined but never used"),
                        suggestion: format!("Remove unused variable '{name}'"),
                    });
                }
            }
            ExprKind::Block(exprs) => {
                for expr in exprs {
                    self.check_unused_variables(expr, issues);
                }
            }
            _ => {}
        }
    }
    
    fn is_variable_used(&self, name: &str, expr: &Expr) -> bool {
        match &expr.kind {
            ExprKind::Identifier(id) => id == name,
            ExprKind::Binary { left, op: _, right } => {
                self.is_variable_used(name, left) || self.is_variable_used(name, right)
            }
            ExprKind::Block(exprs) => {
                exprs.iter().any(|e| self.is_variable_used(name, e))
            }
            _ => false,
        }
    }
    
    fn calculate_complexity(&self, expr: &Expr) -> usize {
        match &expr.kind {
            ExprKind::If { condition: _, then_branch, else_branch, .. } => {
                1 + self.calculate_complexity(then_branch) 
                  + else_branch.as_ref().map_or(0, |e| self.calculate_complexity(e))
            }
            ExprKind::Match { .. } => 2,
            ExprKind::While { .. } | ExprKind::For { .. } => 2,
            ExprKind::Block(exprs) => {
                exprs.iter().map(|e| self.calculate_complexity(e)).sum()
            }
            _ => 0,
        }
    }
}

impl Default for Linter {
    fn default() -> Self {
        Self::new()
    }
}