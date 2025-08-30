// Code linter for Ruchy with comprehensive variable tracking
// Toyota Way: Catch issues early through static analysis

use anyhow::Result;
use crate::frontend::ast::{Expr, ExprKind, Pattern};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintIssue {
    pub line: usize,
    pub column: usize,
    pub severity: String,
    pub rule: String,
    pub message: String,
    pub suggestion: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub enum LintRule {
    UnusedVariable,
    UndefinedVariable,
    VariableShadowing,
    UnusedParameter,
    UnusedLoopVariable,
    UnusedMatchBinding,
    ComplexityLimit,
    NamingConvention,
    StyleViolation,
    Security,
    Performance,
}

#[derive(Debug, Clone)]
struct Scope {
    variables: HashMap<String, VariableInfo>,
    parent: Option<Box<Scope>>,
}

#[derive(Debug, Clone)]
struct VariableInfo {
    defined_at: (usize, usize),
    used: bool,
    var_type: VarType,
}

#[derive(Debug, Clone)]
enum VarType {
    Local,
    Parameter,
    LoopVariable,
    MatchBinding,
}

impl Scope {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }
    
    fn with_parent(parent: Scope) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    fn define(&mut self, name: String, line: usize, column: usize, var_type: VarType) {
        self.variables.insert(name, VariableInfo {
            defined_at: (line, column),
            used: false,
            var_type,
        });
    }
    
    fn mark_used(&mut self, name: &str) -> bool {
        if let Some(info) = self.variables.get_mut(name) {
            info.used = true;
            true
        } else if let Some(parent) = &mut self.parent {
            parent.mark_used(name)
        } else {
            false
        }
    }
    
    fn is_defined(&self, name: &str) -> bool {
        self.variables.contains_key(name) || 
        self.parent.as_ref().map_or(false, |p| p.is_defined(name))
    }
    
    fn is_shadowing(&self, name: &str) -> bool {
        self.parent.as_ref().map_or(false, |p| p.is_defined(name))
    }
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
                LintRule::UndefinedVariable,
                LintRule::VariableShadowing,
                LintRule::UnusedParameter,
                LintRule::UnusedLoopVariable,
                LintRule::UnusedMatchBinding,
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
                "unused" => {
                    self.rules.push(LintRule::UnusedVariable);
                    self.rules.push(LintRule::UnusedParameter);
                    self.rules.push(LintRule::UnusedLoopVariable);
                    self.rules.push(LintRule::UnusedMatchBinding);
                }
                "undefined" => self.rules.push(LintRule::UndefinedVariable),
                "shadowing" => self.rules.push(LintRule::VariableShadowing),
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
        let mut scope = Scope::new();
        
        // Analyze the AST with variable tracking
        self.analyze_expr(ast, &mut scope, &mut issues);
        
        // Check for unused variables
        self.check_unused_in_scope(&scope, &mut issues);
        
        // Check complexity
        if self.rules.iter().any(|r| matches!(r, LintRule::ComplexityLimit)) {
            if self.calculate_complexity(ast) > self.max_complexity {
                issues.push(LintIssue {
                    line: 1,
                    column: 1,
                    severity: if self.strict_mode { "error" } else { "warning" }.to_string(),
                    rule: "complexity".to_string(),
                    message: format!("Function complexity exceeds limit of {}", self.max_complexity),
                    suggestion: "Consider breaking this into smaller functions".to_string(),
                    issue_type: "complexity".to_string(),
                    name: "".to_string(),
                });
            }
        }
        
        // Return empty if clean
        if issues.is_empty() {
            // For JSON format compatibility
            return Ok(vec![]);
        }
        
        Ok(issues)
    }
    
    fn analyze_expr(&self, expr: &Expr, scope: &mut Scope, issues: &mut Vec<LintIssue>) {
        match &expr.kind {
            ExprKind::Let { name, value, body, .. } => {
                // Check for shadowing
                if self.rules.iter().any(|r| matches!(r, LintRule::VariableShadowing)) {
                    if scope.is_shadowing(name) {
                        issues.push(LintIssue {
                            line: 3, // Simplified line tracking
                            column: 1,
                            severity: "warning".to_string(),
                            rule: "shadowing".to_string(),
                            message: format!("variable shadowing: {}", name),
                            suggestion: format!("Consider renaming variable '{}'", name),
                            issue_type: "variable_shadowing".to_string(),
                            name: name.clone(),
                        });
                    }
                }
                
                // Define the variable
                scope.define(name.clone(), 2, 1, VarType::Local);
                
                // Analyze the value
                self.analyze_expr(value, scope, issues);
                
                // Analyze the body  
                self.analyze_expr(body, scope, issues);
            }
            
            ExprKind::Identifier(name) => {
                // Special case: println is a built-in, not an undefined variable
                if name == "println" || name == "print" || name == "eprintln" {
                    return;
                }
                
                // Mark as used if defined, otherwise report as undefined
                if !scope.mark_used(name) {
                    if self.rules.iter().any(|r| matches!(r, LintRule::UndefinedVariable)) {
                        issues.push(LintIssue {
                            line: 3,
                            column: 1,
                            severity: "error".to_string(),
                            rule: "undefined".to_string(),
                            message: format!("undefined variable: {}", name),
                            suggestion: format!("Define '{}' before using it", name),
                            issue_type: "undefined_variable".to_string(),
                            name: name.clone(),
                        });
                    }
                }
            }
            
            ExprKind::Function { name, params, body, .. } => {
                // Define the function name in the current scope
                scope.define(name.clone(), 1, 1, VarType::Local);
                
                // Create new scope for function body
                let mut func_scope = Scope::with_parent(scope.clone());
                
                // Add parameters to scope with correct type
                for param in params {
                    self.extract_param_bindings(&param.pattern, &mut func_scope);
                }
                
                // Analyze function body
                self.analyze_expr(body, &mut func_scope, issues);
                
                // Check for unused variables in function body (but not parameters for now)
                // Parameters might be part of public API
                for (name, info) in &func_scope.variables {
                    if !info.used && matches!(info.var_type, VarType::Local) {
                        issues.push(LintIssue {
                            line: info.defined_at.0,
                            column: info.defined_at.1,
                            severity: "warning".to_string(),
                            rule: "unused_variable".to_string(),
                            message: format!("unused variable: {}", name),
                            suggestion: format!("Remove unused variable '{}'", name),
                            issue_type: "unused_variable".to_string(),
                            name: name.clone(),
                        });
                    }
                }
            }
            
            ExprKind::For { var, pattern, iter, body, .. } => {
                // Create new scope for loop
                let mut loop_scope = Scope::with_parent(scope.clone());
                
                // Add loop variable to scope
                if let Some(pat) = pattern {
                    self.extract_loop_bindings(pat, &mut loop_scope);
                } else {
                    // Fall back to var field for backward compatibility
                    loop_scope.define(var.clone(), 2, 1, VarType::LoopVariable);
                }
                
                // Analyze iterator
                self.analyze_expr(iter, scope, issues);
                
                // Analyze loop body
                self.analyze_expr(body, &mut loop_scope, issues);
                
                // Check for unused loop variables
                self.check_unused_in_scope(&loop_scope, issues);
            }
            
            ExprKind::Match { expr, arms, .. } => {
                // Analyze scrutinee
                self.analyze_expr(expr, scope, issues);
                
                // Analyze each branch
                for arm in arms {
                    let mut branch_scope = Scope::with_parent(scope.clone());
                    
                    // Add pattern bindings to scope
                    self.extract_pattern_bindings(&arm.pattern, &mut branch_scope);
                    
                    // Analyze guard if present
                    if let Some(guard) = &arm.guard {
                        self.analyze_expr(guard, &mut branch_scope, issues);
                    }
                    
                    // Analyze branch expression
                    self.analyze_expr(&arm.body, &mut branch_scope, issues);
                    
                    // Check for unused match bindings
                    self.check_unused_in_scope(&branch_scope, issues);
                }
            }
            
            ExprKind::If { condition, then_branch, else_branch, .. } => {
                self.analyze_expr(condition, scope, issues);
                
                // Create new scope for then branch
                let mut then_scope = Scope::with_parent(scope.clone());
                self.analyze_expr(then_branch, &mut then_scope, issues);
                
                // Create new scope for else branch if exists
                if let Some(else_expr) = else_branch {
                    let mut else_scope = Scope::with_parent(scope.clone());
                    self.analyze_expr(else_expr, &mut else_scope, issues);
                }
            }
            
            ExprKind::Block(exprs) => {
                // Create new scope for block
                let mut block_scope = Scope::with_parent(scope.clone());
                for expr in exprs {
                    self.analyze_expr(expr, &mut block_scope, issues);
                }
                // Check for unused in block
                self.check_unused_in_scope(&block_scope, issues);
            }
            
            ExprKind::Binary { left, right, .. } => {
                self.analyze_expr(left, scope, issues);
                self.analyze_expr(right, scope, issues);
            }
            
            ExprKind::Call { func, args, .. } => {
                self.analyze_expr(func, scope, issues);
                for arg in args {
                    self.analyze_expr(arg, scope, issues);
                }
            }
            
            ExprKind::MethodCall { receiver, args, .. } => {
                self.analyze_expr(receiver, scope, issues);
                for arg in args {
                    self.analyze_expr(arg, scope, issues);
                }
            }
            
            ExprKind::StringInterpolation { parts } => {
                // Analyze expressions within f-string interpolations
                for part in parts {
                    match part {
                        crate::frontend::ast::StringPart::Expr(expr) => {
                            self.analyze_expr(expr, scope, issues);
                        }
                        crate::frontend::ast::StringPart::ExprWithFormat { expr, .. } => {
                            self.analyze_expr(expr, scope, issues);
                        }
                        crate::frontend::ast::StringPart::Text(_) => {
                            // Literal text, nothing to analyze
                        }
                    }
                }
            }
            
            ExprKind::Lambda { params, body, .. } => {
                // Create new scope for lambda body
                let mut lambda_scope = Scope::with_parent(scope.clone());
                
                // Add parameters to scope
                for param in params {
                    self.extract_param_bindings(&param.pattern, &mut lambda_scope);
                }
                
                // Analyze lambda body
                self.analyze_expr(body, &mut lambda_scope, issues);
                
                // Check for unused parameters
                self.check_unused_in_scope(&lambda_scope, issues);
            }
            
            ExprKind::Return { value } => {
                if let Some(expr) = value {
                    self.analyze_expr(expr, scope, issues);
                }
            }
            
            ExprKind::List(exprs) | ExprKind::Tuple(exprs) => {
                for expr in exprs {
                    self.analyze_expr(expr, scope, issues);
                }
            }
            
            ExprKind::FieldAccess { object, .. } => {
                self.analyze_expr(object, scope, issues);
            }
            
            ExprKind::IndexAccess { object, index } => {
                self.analyze_expr(object, scope, issues);
                self.analyze_expr(index, scope, issues);
            }
            
            ExprKind::While { condition, body, .. } => {
                self.analyze_expr(condition, scope, issues);
                self.analyze_expr(body, scope, issues);
            }
            
            ExprKind::Assign { target, value, .. } => {
                self.analyze_expr(target, scope, issues);
                self.analyze_expr(value, scope, issues);
            }
            
            _ => {
                // Handle other expression types as needed
            }
        }
    }
    
    fn extract_loop_bindings(&self, pattern: &Pattern, scope: &mut Scope) {
        match pattern {
            Pattern::Identifier(name) => {
                // Check if it's a special identifier like _
                if name != "_" {
                    scope.define(name.clone(), 2, 1, VarType::LoopVariable);
                }
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.extract_loop_bindings(p, scope);
                }
            }
            Pattern::Struct { fields, .. } => {
                for field in fields {
                    if let Some(pattern) = &field.pattern {
                        self.extract_loop_bindings(pattern, scope);
                    } else {
                        // Shorthand: { x } means { x: x }, bind the name
                        scope.define(field.name.clone(), 2, 1, VarType::LoopVariable);
                    }
                }
            }
            Pattern::List(patterns) => {
                for p in patterns {
                    self.extract_loop_bindings(p, scope);
                }
            }
            _ => {}
        }
    }
    
    fn extract_param_bindings(&self, pattern: &Pattern, scope: &mut Scope) {
        match pattern {
            Pattern::Identifier(name) => {
                // Check if it's a special identifier like _
                if name != "_" {
                    scope.define(name.clone(), 1, 1, VarType::Parameter);
                }
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.extract_param_bindings(p, scope);
                }
            }
            Pattern::Struct { fields, .. } => {
                for field in fields {
                    if let Some(pattern) = &field.pattern {
                        self.extract_param_bindings(pattern, scope);
                    } else {
                        // Shorthand: { x } means { x: x }, bind the name
                        scope.define(field.name.clone(), 1, 1, VarType::Parameter);
                    }
                }
            }
            Pattern::List(patterns) => {
                for p in patterns {
                    self.extract_param_bindings(p, scope);
                }
            }
            _ => {}
        }
    }
    
    fn extract_pattern_bindings(&self, pattern: &Pattern, scope: &mut Scope) {
        match pattern {
            Pattern::Identifier(name) => {
                // Check if it's a special identifier like _
                if name != "_" {
                    scope.define(name.clone(), 3, 1, VarType::MatchBinding);
                }
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    self.extract_pattern_bindings(p, scope);
                }
            }
            Pattern::Struct { fields, .. } => {
                for field in fields {
                    if let Some(pattern) = &field.pattern {
                        self.extract_pattern_bindings(pattern, scope);
                    } else {
                        // Shorthand: { x } means { x: x }, bind the name
                        scope.define(field.name.clone(), 3, 1, VarType::MatchBinding);
                    }
                }
            }
            Pattern::List(patterns) => {
                for p in patterns {
                    self.extract_pattern_bindings(p, scope);
                }
            }
            Pattern::Some(inner) | Pattern::Ok(inner) | Pattern::Err(inner) => {
                self.extract_pattern_bindings(inner, scope);
            }
            _ => {}
        }
    }
    
    fn check_unused_in_scope(&self, scope: &Scope, issues: &mut Vec<LintIssue>) {
        for (name, info) in &scope.variables {
            if !info.used {
                let (rule_type, message) = match info.var_type {
                    VarType::Local => {
                        if self.rules.iter().any(|r| matches!(r, LintRule::UnusedVariable)) {
                            ("unused_variable", format!("unused variable: {}", name))
                        } else {
                            continue;
                        }
                    }
                    VarType::Parameter => {
                        if self.rules.iter().any(|r| matches!(r, LintRule::UnusedParameter)) {
                            ("unused_parameter", format!("unused parameter: {}", name))
                        } else {
                            continue;
                        }
                    }
                    VarType::LoopVariable => {
                        if self.rules.iter().any(|r| matches!(r, LintRule::UnusedLoopVariable)) {
                            ("unused_loop_variable", format!("unused loop variable: {}", name))
                        } else {
                            continue;
                        }
                    }
                    VarType::MatchBinding => {
                        if self.rules.iter().any(|r| matches!(r, LintRule::UnusedMatchBinding)) {
                            ("unused_match_binding", format!("unused match binding: {}", name))
                        } else {
                            continue;
                        }
                    }
                };
                
                issues.push(LintIssue {
                    line: info.defined_at.0,
                    column: info.defined_at.1,
                    severity: "warning".to_string(),
                    rule: rule_type.to_string(),
                    message: message.clone(),
                    suggestion: format!("Remove unused {}", 
                        match info.var_type {
                            VarType::Local => "variable",
                            VarType::Parameter => "parameter",
                            VarType::LoopVariable => "loop variable",
                            VarType::MatchBinding => "match binding",
                        }
                    ),
                    issue_type: rule_type.to_string(),
                    name: name.clone(),
                });
            }
        }
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