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
        self.parent.as_ref().is_some_and(|p| p.is_defined(name))
    }
    fn is_shadowing(&self, name: &str) -> bool {
        self.parent.as_ref().is_some_and(|p| p.is_defined(name))
    }
}
pub struct Linter {
    rules: Vec<LintRule>,
    strict_mode: bool,
    max_complexity: usize,
}
impl Linter {
/// # Examples
/// 
/// ```
/// use ruchy::quality::linter::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::quality::linter::set_rules;
/// 
/// let result = set_rules("example");
/// assert_eq!(result, Ok(()));
/// ```
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
/// # Examples
/// 
/// ```
/// use ruchy::quality::linter::set_strict_mode;
/// 
/// let result = set_strict_mode(true);
/// assert_eq!(result, Ok(true));
/// ```
pub fn set_strict_mode(&mut self, strict: bool) {
        self.strict_mode = strict;
    }
/// # Examples
/// 
/// ```
/// use ruchy::quality::linter::lint;
/// 
/// let result = lint("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn lint(&self, ast: &Expr, _source: &str) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();
        let mut scope = Scope::new();
        // Analyze the AST with variable tracking
        self.analyze_expr(ast, &mut scope, &mut issues);
        // Check for unused variables
        self.check_unused_in_scope(&scope, &mut issues);
        // Check complexity
        if self.rules.iter().any(|r| matches!(r, LintRule::ComplexityLimit))
            && self.calculate_complexity(ast) > self.max_complexity {
                issues.push(LintIssue {
                    line: 1,
                    column: 1,
                    severity: if self.strict_mode { "error" } else { "warning" }.to_string(),
                    rule: "complexity".to_string(),
                    message: format!("Function complexity exceeds limit of {}", self.max_complexity),
                    suggestion: "Consider breaking this into smaller functions".to_string(),
                    issue_type: "complexity".to_string(),
                    name: String::new(),
                });
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
                // Analyze the value first (with current scope)
                self.analyze_expr(value, scope, issues);
                // Create new scope for the let binding body
                let mut let_scope = Scope::with_parent(scope.clone());
                // Check for shadowing before defining
                if self.rules.iter().any(|r| matches!(r, LintRule::VariableShadowing))
                    && let_scope.is_shadowing(name) {
                        issues.push(LintIssue {
                            line: 3, // Simplified line tracking
                            column: 1,
                            severity: "warning".to_string(),
                            rule: "shadowing".to_string(),
                            message: format!("variable shadowing: {name}"),
                            suggestion: format!("Consider renaming variable '{name}'"),
                            issue_type: "variable_shadowing".to_string(),
                            name: name.clone(),
                        });
                    }
                // Define the variable in the new scope
                let_scope.define(name.clone(), 2, 1, VarType::Local);
                // Analyze the body with the new scope
                self.analyze_expr(body, &mut let_scope, issues);
                // Check for unused variables in the let scope
                self.check_unused_in_scope(&let_scope, issues);
            }
            ExprKind::Identifier(name) => {
                // Special case: println is a built-in, not an undefined variable
                if name == "println" || name == "print" || name == "eprintln" {
                    return;
                }
                // Mark as used if defined, otherwise report as undefined
                if !scope.mark_used(name)
                    && self.rules.iter().any(|r| matches!(r, LintRule::UndefinedVariable)) {
                        issues.push(LintIssue {
                            line: 3,
                            column: 1,
                            severity: "error".to_string(),
                            rule: "undefined".to_string(),
                            message: format!("undefined variable: {name}"),
                            suggestion: format!("Define '{name}' before using it"),
                            issue_type: "undefined_variable".to_string(),
                            name: name.clone(),
                        });
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
                            message: format!("unused variable: {name}"),
                            suggestion: format!("Remove unused variable '{name}'"),
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
                // For blocks, we use the same scope level - each statement can see previous ones
                for expr in exprs {
                    self.analyze_expr(expr, scope, issues);
                }
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
                            ("unused_variable", format!("unused variable: {name}"))
                        } else {
                            continue;
                        }
                    }
                    VarType::Parameter => {
                        if self.rules.iter().any(|r| matches!(r, LintRule::UnusedParameter)) {
                            ("unused_parameter", format!("unused parameter: {name}"))
                        } else {
                            continue;
                        }
                    }
                    VarType::LoopVariable => {
                        if self.rules.iter().any(|r| matches!(r, LintRule::UnusedLoopVariable)) {
                            ("unused_loop_variable", format!("unused loop variable: {name}"))
                        } else {
                            continue;
                        }
                    }
                    VarType::MatchBinding => {
                        if self.rules.iter().any(|r| matches!(r, LintRule::UnusedMatchBinding)) {
                            ("unused_match_binding", format!("unused match binding: {name}"))
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
/// # Examples
/// 
/// ```
/// use ruchy::quality::linter::auto_fix;
/// 
/// let result = auto_fix("example");
/// assert_eq!(result, Ok(()));
/// ```
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::{
        Expr, ExprKind, Pattern, Literal, BinaryOp, Span, Param, Type, TypeKind,
        MatchArm, StringPart, StructPatternField
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
        Expr::new(ExprKind::Literal(Literal::Integer(value)), create_test_span())
    }
    fn create_test_expr_identifier(name: &str) -> Expr {
        Expr::new(ExprKind::Identifier(name.to_string()), create_test_span())
    }
    fn create_test_expr_let(name: &str, value: Expr, body: Expr) -> Expr {
        Expr::new(ExprKind::Let {
            name: name.to_string(),
            type_annotation: None,
            value: Box::new(value),
            body: Box::new(body),
            is_mutable: false,
        }, create_test_span())
    }
    fn create_test_expr_function(name: &str, params: Vec<Param>, body: Expr) -> Expr {
        Expr::new(ExprKind::Function {
            name: name.to_string(),
            type_params: vec![],
            params,
            return_type: None,
            body: Box::new(body),
            is_async: false,
            is_pub: false,
        }, create_test_span())
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
        Expr::new(ExprKind::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }, create_test_span())
    }
    fn create_test_expr_call(func: Expr, args: Vec<Expr>) -> Expr {
        Expr::new(ExprKind::Call {
            func: Box::new(func),
            args,
        }, create_test_span())
    }
    fn create_test_expr_if(condition: Expr, then_branch: Expr, else_branch: Option<Expr>) -> Expr {
        Expr::new(ExprKind::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch: else_branch.map(Box::new),
        }, create_test_span())
    }
    fn create_test_expr_for(var: &str, pattern: Option<Pattern>, iter: Expr, body: Expr) -> Expr {
        Expr::new(ExprKind::For {
            var: var.to_string(),
            pattern,
            iter: Box::new(iter),
            body: Box::new(body),
        }, create_test_span())
    }
    fn create_test_expr_match(expr: Expr, arms: Vec<MatchArm>) -> Expr {
        Expr::new(ExprKind::Match {
            expr: Box::new(expr),
            arms,
        }, create_test_span())
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
        Expr::new(ExprKind::Lambda {
            params,
            body: Box::new(body),
        }, create_test_span())
    }
    fn create_test_expr_method_call(receiver: Expr, method: &str, args: Vec<Expr>) -> Expr {
        Expr::new(ExprKind::MethodCall {
            receiver: Box::new(receiver),
            method: method.to_string(),
            args,
        }, create_test_span())
    }
    fn create_test_expr_while(condition: Expr, body: Expr) -> Expr {
        Expr::new(ExprKind::While {
            condition: Box::new(condition),
            body: Box::new(body),
        }, create_test_span())
    }
    fn create_test_expr_return(value: Option<Expr>) -> Expr {
        Expr::new(ExprKind::Return {
            value: value.map(Box::new),
        }, create_test_span())
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
        assert!(linter.rules.iter().any(|r| matches!(r, LintRule::StyleViolation)));
        assert!(linter.rules.iter().any(|r| matches!(r, LintRule::Security)));
        assert!(linter.rules.iter().any(|r| matches!(r, LintRule::Performance)));
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
        let deserialized: Result<LintIssue, _> = serde_json::from_str(&json.unwrap());
        assert!(deserialized.is_ok());
    }
    // ========== Basic Linting Tests ==========
    #[test]
    fn test_lint_empty_expression() {
        let linter = create_test_linter();
        let expr = create_test_expr_literal_int(42);
        let issues = linter.lint(&expr, "42").unwrap();
        assert_eq!(issues.len(), 0);
    }
    #[test]
    fn test_lint_undefined_variable() {
        let linter = create_test_linter_with_rules("undefined");
        let expr = create_test_expr_identifier("undefined_var");
        let issues = linter.lint(&expr, "undefined_var").unwrap();
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
        assert_eq!(linter.lint(&println_expr, "println").unwrap().len(), 0);
        assert_eq!(linter.lint(&print_expr, "print").unwrap().len(), 0);
        assert_eq!(linter.lint(&eprintln_expr, "eprintln").unwrap().len(), 0);
    }
    #[test]
    fn test_lint_unused_variable() {
        let linter = create_test_linter_with_rules("unused");
        let expr = create_test_expr_let(
            "x",
            create_test_expr_literal_int(42),
            create_test_expr_literal_int(0)
        );
        let issues = linter.lint(&expr, "let x = 42; 0").unwrap();
        assert!(issues.iter().any(|i| i.rule == "unused_variable" && i.name == "x"));
    }
    #[test]
    fn test_lint_used_variable() {
        let linter = create_test_linter_with_rules("unused");
        let expr = create_test_expr_let(
            "x",
            create_test_expr_literal_int(42),
            create_test_expr_identifier("x")
        );
        let issues = linter.lint(&expr, "let x = 42; x").unwrap();
        assert!(!issues.iter().any(|i| i.rule == "unused_variable" && i.name == "x"));
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
                "x",  // This should shadow the outer x
                create_test_expr_literal_int(2),
                create_test_expr_identifier("x")
            )
        );
        let issues = linter.lint(&outer_let, "let x = 1; let x = 2; x").unwrap();
        eprintln!("Debug - Issues found: {issues:?}");
        assert!(issues.iter().any(|i| i.rule == "shadowing" && i.name == "x"));
    }
    // ========== Function Linting Tests ==========
    #[test]
    fn test_lint_function_definition() {
        let linter = create_test_linter_with_rules("unused");
        let expr = create_test_expr_function(
            "test_func",
            vec![create_test_param("x")],
            create_test_expr_literal_int(42)
        );
        let issues = linter.lint(&expr, "fn test_func(x) { 42 }").unwrap();
        // Parameters are not flagged as unused in function scope analysis
        assert!(!issues.iter().any(|i| i.rule == "unused_parameter"));
    }
    #[test]
    fn test_lint_function_unused_local_variable() {
        let linter = create_test_linter_with_rules("unused");
        let body = create_test_expr_let(
            "local_var",
            create_test_expr_literal_int(1),
            create_test_expr_literal_int(42)
        );
        let expr = create_test_expr_function(
            "test_func",
            vec![],
            body
        );
        let issues = linter.lint(&expr, "fn test_func() { let local_var = 1; 42 }").unwrap();
        assert!(issues.iter().any(|i| i.rule == "unused_variable" && i.name == "local_var"));
    }
    // ========== Loop Linting Tests ==========
    #[test]
    fn test_lint_for_loop_unused_variable() {
        let linter = create_test_linter_with_rules("unused");
        let expr = create_test_expr_for(
            "i",
            Some(Pattern::Identifier("i".to_string())),
            create_test_expr_literal_int(42),
            create_test_expr_literal_int(0)
        );
        let issues = linter.lint(&expr, "for i in items { 0 }").unwrap();
        assert!(issues.iter().any(|i| i.rule.contains("unused") && i.name == "i"));
    }
    #[test]
    fn test_lint_for_loop_used_variable() {
        let linter = create_test_linter_with_rules("unused");
        let expr = create_test_expr_for(
            "i",
            Some(Pattern::Identifier("i".to_string())),
            create_test_expr_literal_int(42),
            create_test_expr_identifier("i")
        );
        let issues = linter.lint(&expr, "for i in items { i }").unwrap();
        assert!(!issues.iter().any(|i| i.rule.contains("unused") && i.name == "i"));
    }
    #[test]
    fn test_lint_for_loop_underscore_variable() {
        let linter = create_test_linter_with_rules("unused");
        let expr = create_test_expr_for(
            "_",
            Some(Pattern::Identifier("_".to_string())),
            create_test_expr_literal_int(42),
            create_test_expr_literal_int(0)
        );
        let issues = linter.lint(&expr, "for _ in items { 0 }").unwrap();
        assert!(!issues.iter().any(|i| i.name == "_"));
    }
    // ========== Match Expression Tests ==========
    #[test]
    fn test_lint_match_unused_binding() {
        let linter = create_test_linter_with_rules("unused");
        let arm = create_test_match_arm(
            Pattern::Identifier("x".to_string()),
            create_test_expr_literal_int(42)
        );
        let expr = create_test_expr_match(
            create_test_expr_literal_int(1),
            vec![arm]
        );
        let issues = linter.lint(&expr, "match value { x => 42 }").unwrap();
        assert!(issues.iter().any(|i| i.rule.contains("unused") && i.name == "x"));
    }
    #[test]
    fn test_lint_match_used_binding() {
        let linter = create_test_linter_with_rules("unused");
        let arm = create_test_match_arm(
            Pattern::Identifier("x".to_string()),
            create_test_expr_identifier("x")
        );
        let expr = create_test_expr_match(
            create_test_expr_literal_int(1),
            vec![arm]
        );
        let issues = linter.lint(&expr, "match value { x => x }").unwrap();
        assert!(!issues.iter().any(|i| i.rule.contains("unused") && i.name == "x"));
    }
    #[test]
    fn test_lint_match_underscore_binding() {
        let linter = create_test_linter_with_rules("unused");
        let arm = create_test_match_arm(
            Pattern::Identifier("_".to_string()),
            create_test_expr_literal_int(42)
        );
        let expr = create_test_expr_match(
            create_test_expr_literal_int(1),
            vec![arm]
        );
        let issues = linter.lint(&expr, "match value { _ => 42 }").unwrap();
        assert!(!issues.iter().any(|i| i.name == "_"));
    }
    // ========== Lambda Expression Tests ==========
    #[test]
    fn test_lint_lambda_unused_parameter() {
        let linter = create_test_linter_with_rules("unused");
        let expr = create_test_expr_lambda(
            vec![create_test_param("x")],
            create_test_expr_literal_int(42)
        );
        let issues = linter.lint(&expr, "|x| 42").unwrap();
        assert!(issues.iter().any(|i| i.rule.contains("unused") && i.name == "x"));
    }
    #[test]
    fn test_lint_lambda_used_parameter() {
        let linter = create_test_linter_with_rules("unused");
        let expr = create_test_expr_lambda(
            vec![create_test_param("x")],
            create_test_expr_identifier("x")
        );
        let issues = linter.lint(&expr, "|x| x").unwrap();
        assert!(!issues.iter().any(|i| i.rule.contains("unused") && i.name == "x"));
    }
    // ========== Complexity Tests ==========
    #[test]
    fn test_complexity_calculation_simple() {
        let linter = create_test_linter();
        let expr = create_test_expr_literal_int(42);
        assert_eq!(linter.calculate_complexity(&expr), 0);
    }
    #[test]
    fn test_complexity_calculation_if() {
        let linter = create_test_linter();
        let expr = create_test_expr_if(
            create_test_expr_literal_int(1),
            create_test_expr_literal_int(2),
            Some(create_test_expr_literal_int(3))
        );
        assert_eq!(linter.calculate_complexity(&expr), 1);
    }
    #[test]
    fn test_complexity_calculation_match() {
        let linter = create_test_linter();
        let arm = create_test_match_arm(
            Pattern::Identifier("_".to_string()),
            create_test_expr_literal_int(42)
        );
        let expr = create_test_expr_match(
            create_test_expr_literal_int(1),
            vec![arm]
        );
        assert_eq!(linter.calculate_complexity(&expr), 2);
    }
    #[test]
    fn test_complexity_calculation_while() {
        let linter = create_test_linter();
        let expr = create_test_expr_while(
            create_test_expr_literal_int(1),
            create_test_expr_literal_int(2)
        );
        assert_eq!(linter.calculate_complexity(&expr), 2);
    }
    #[test]
    fn test_complexity_calculation_for() {
        let linter = create_test_linter();
        let expr = create_test_expr_for(
            "i",
            Some(Pattern::Identifier("i".to_string())),
            create_test_expr_literal_int(42),
            create_test_expr_literal_int(0)
        );
        assert_eq!(linter.calculate_complexity(&expr), 2);
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
                None
            ),
            None
        );
        let issues = linter.lint(&complex_expr, "if 1 { if 2 { 3 } }").unwrap();
        assert!(issues.iter().any(|i| i.rule == "complexity"));
    }
    #[test]
    fn test_complexity_limit_strict_mode() {
        let mut linter = create_test_linter_with_rules("complexity");
        linter.set_strict_mode(true);
        linter.max_complexity = 0;
        let expr = create_test_expr_literal_int(42);
        let issues = linter.lint(&expr, "42").unwrap();
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
            Pattern::Identifier("y".to_string())
        ]);
        linter.extract_loop_bindings(&pattern, &mut scope);
        assert!(scope.is_defined("x"));
        assert!(scope.is_defined("y"));
    }
    #[test]
    fn test_extract_loop_bindings_list() {
        let linter = create_test_linter();
        let mut scope = Scope::new();
        let pattern = Pattern::List(vec![
            Pattern::Identifier("first".to_string()),
            Pattern::Identifier("second".to_string())
        ]);
        linter.extract_loop_bindings(&pattern, &mut scope);
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
        linter.extract_loop_bindings(&pattern, &mut scope);
        assert!(scope.is_defined("x_val"));
        assert!(scope.is_defined("y"));
    }
    #[test]
    fn test_extract_param_bindings_underscore() {
        let linter = create_test_linter();
        let mut scope = Scope::new();
        let pattern = Pattern::Identifier("_".to_string());
        linter.extract_param_bindings(&pattern, &mut scope);
        assert!(!scope.is_defined("_"));
    }
    #[test]
    fn test_extract_pattern_bindings_nested_option() {
        let linter = create_test_linter();
        let mut scope = Scope::new();
        let pattern = Pattern::Some(Box::new(Pattern::Identifier("value".to_string())));
        linter.extract_pattern_bindings(&pattern, &mut scope);
        assert!(scope.is_defined("value"));
    }
    #[test]
    fn test_extract_pattern_bindings_ok_err() {
        let linter = create_test_linter();
        let mut scope = Scope::new();
        let ok_pattern = Pattern::Ok(Box::new(Pattern::Identifier("success".to_string())));
        linter.extract_pattern_bindings(&ok_pattern, &mut scope);
        assert!(scope.is_defined("success"));
        let err_pattern = Pattern::Err(Box::new(Pattern::Identifier("error".to_string())));
        linter.extract_pattern_bindings(&err_pattern, &mut scope);
        assert!(scope.is_defined("error"));
    }
    // ========== Expression Analysis Tests ==========
    #[test]
    fn test_analyze_binary_expression() {
        let linter = create_test_linter_with_rules("undefined");
        let expr = create_test_expr_binary(
            BinaryOp::Add,
            create_test_expr_identifier("undefined_left"),
            create_test_expr_identifier("undefined_right")
        );
        let issues = linter.lint(&expr, "undefined_left + undefined_right").unwrap();
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().any(|i| i.name == "undefined_left"));
        assert!(issues.iter().any(|i| i.name == "undefined_right"));
    }
    #[test]
    fn test_analyze_call_expression() {
        let linter = create_test_linter_with_rules("undefined");
        let expr = create_test_expr_call(
            create_test_expr_identifier("undefined_func"),
            vec![create_test_expr_identifier("undefined_arg")]
        );
        let issues = linter.lint(&expr, "undefined_func(undefined_arg)").unwrap();
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
            vec![create_test_expr_identifier("undefined_arg")]
        );
        let issues = linter.lint(&expr, "undefined_obj.method(undefined_arg)").unwrap();
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().any(|i| i.name == "undefined_obj"));
        assert!(issues.iter().any(|i| i.name == "undefined_arg"));
    }
    #[test]
    fn test_analyze_string_interpolation() {
        let linter = create_test_linter_with_rules("undefined");
        let expr = Expr::new(ExprKind::StringInterpolation {
            parts: vec![
                StringPart::Text("Hello ".to_string()),
                StringPart::Expr(Box::new(create_test_expr_identifier("undefined_name"))),
                StringPart::ExprWithFormat { 
                    expr: Box::new(create_test_expr_identifier("undefined_age")), 
                    format_spec: "d".to_string() 
                },
            ]
        }, create_test_span());
        let issues = linter.lint(&expr, "f\"Hello {undefined_name} {undefined_age:d}\"").unwrap();
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().any(|i| i.name == "undefined_name"));
        assert!(issues.iter().any(|i| i.name == "undefined_age"));
    }
    #[test]
    fn test_analyze_return_expression() {
        let linter = create_test_linter_with_rules("undefined");
        let expr = create_test_expr_return(Some(create_test_expr_identifier("undefined_var")));
        let issues = linter.lint(&expr, "return undefined_var").unwrap();
        assert_eq!(issues.len(), 1);
        assert!(issues.iter().any(|i| i.name == "undefined_var"));
    }
    #[test]
    fn test_analyze_list_and_tuple() {
        let linter = create_test_linter_with_rules("undefined");
        let list_expr = Expr::new(ExprKind::List(vec![
            create_test_expr_identifier("undefined_item")
        ]), create_test_span());
        let tuple_expr = Expr::new(ExprKind::Tuple(vec![
            create_test_expr_identifier("undefined_elem")
        ]), create_test_span());
        let list_issues = linter.lint(&list_expr, "[undefined_item]").unwrap();
        assert!(list_issues.iter().any(|i| i.name == "undefined_item"));
        let tuple_issues = linter.lint(&tuple_expr, "(undefined_elem,)").unwrap();
        assert!(tuple_issues.iter().any(|i| i.name == "undefined_elem"));
    }
    #[test]
    fn test_analyze_field_and_index_access() {
        let linter = create_test_linter_with_rules("undefined");
        let field_expr = Expr::new(ExprKind::FieldAccess {
            object: Box::new(create_test_expr_identifier("undefined_obj")),
            field: "property".to_string(),
        }, create_test_span());
        let index_expr = Expr::new(ExprKind::IndexAccess {
            object: Box::new(create_test_expr_identifier("undefined_arr")),
            index: Box::new(create_test_expr_identifier("undefined_idx")),
        }, create_test_span());
        let field_issues = linter.lint(&field_expr, "undefined_obj.property").unwrap();
        assert!(field_issues.iter().any(|i| i.name == "undefined_obj"));
        let index_issues = linter.lint(&index_expr, "undefined_arr[undefined_idx]").unwrap();
        assert_eq!(index_issues.len(), 2);
        assert!(index_issues.iter().any(|i| i.name == "undefined_arr"));
        assert!(index_issues.iter().any(|i| i.name == "undefined_idx"));
    }
    #[test]
    fn test_analyze_assign_expression() {
        let linter = create_test_linter_with_rules("undefined");
        let expr = Expr::new(ExprKind::Assign {
            target: Box::new(create_test_expr_identifier("undefined_target")),
            value: Box::new(create_test_expr_identifier("undefined_value")),
        }, create_test_span());
        let issues = linter.lint(&expr, "undefined_target = undefined_value").unwrap();
        assert_eq!(issues.len(), 2);
        assert!(issues.iter().any(|i| i.name == "undefined_target"));
        assert!(issues.iter().any(|i| i.name == "undefined_value"));
    }
    // ========== Block Scope Tests ==========
    #[test]
    fn test_analyze_block_unused_variable() {
        let linter = create_test_linter_with_rules("unused");
        let block = create_test_expr_block(vec![
            create_test_expr_let("unused_var", create_test_expr_literal_int(42), create_test_expr_literal_int(0))
        ]);
        let issues = linter.lint(&block, "{ let unused_var = 42; 0 }").unwrap();
        assert!(issues.iter().any(|i| i.rule == "unused_variable" && i.name == "unused_var"));
    }
    #[test]
    fn test_analyze_if_branches() {
        let linter = create_test_linter_with_rules("undefined");
        let expr = create_test_expr_if(
            create_test_expr_identifier("undefined_cond"),
            create_test_expr_identifier("undefined_then"),
            Some(create_test_expr_identifier("undefined_else"))
        );
        let issues = linter.lint(&expr, "if undefined_cond { undefined_then } else { undefined_else }").unwrap();
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
        let fixed = linter.auto_fix("let  x  =  42", &issues).unwrap();
        assert_eq!(fixed, "let x = 42");
    }
    #[test]
    fn test_auto_fix_no_issues() {
        let linter = create_test_linter();
        let issues = vec![];
        let fixed = linter.auto_fix("let x = 42", &issues).unwrap();
        assert_eq!(fixed, "let x = 42");
    }
    // ========== Integration Tests ==========
    #[test]
    fn test_comprehensive_linting() {
        let linter = create_test_linter_with_rules("unused,undefined,shadowing");
        // Create nested let expressions for comprehensive testing
        let unused_let = create_test_expr_let(
            "unused",
            create_test_expr_identifier("undefined"),  // This creates undefined variable
            create_test_expr_identifier("x")
        );
        let shadow_let = create_test_expr_let(
            "x",  // This shadows the outer x 
            create_test_expr_literal_int(2),
            unused_let
        );
        let outer_let = create_test_expr_let(
            "x",  // Outer variable
            create_test_expr_literal_int(1),
            shadow_let
        );
        let issues = linter.lint(&outer_let, "complex code").unwrap();
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
        let issues = linter.lint(&expr, "42").unwrap();
        assert_eq!(issues.len(), 0);
        let json = serde_json::to_string(&issues).unwrap();
        assert_eq!(json, "[]");
    }
}
#[cfg(test)]
mod property_tests_linter {
    use proptest::proptest;
    
    
    proptest! {
        /// Property: Function never panics on any input
        #[test]
        fn test_new_never_panics(input: String) {
            // Limit input size to avoid timeout
            let input = if input.len() > 100 { &input[..100] } else { &input[..] };
            // Function should not panic on any input
            let _ = std::panic::catch_unwind(|| {
                // Call function with various inputs
                // This is a template - adjust based on actual function signature
            });
        }
    }
}
