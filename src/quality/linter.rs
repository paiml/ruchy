// Code linter for Ruchy with comprehensive variable tracking
// Toyota Way: Catch issues early through static analysis
use crate::frontend::ast::{Expr, ExprKind, Literal, Pattern};
use anyhow::Result;
use serde::{Deserialize, Serialize};
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
    Function, // Function definitions (not checked for unused)
    LoopVariable,
    MatchBinding,
    TypeName, // Enum/Struct type names (Issue #107 fix)
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
        self.variables.insert(
            name,
            VariableInfo {
                defined_at: (line, column),
                used: false,
                var_type,
            },
        );
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
        self.variables.contains_key(name)
            || self.parent.as_ref().is_some_and(|p| p.is_defined(name))
    }
    fn is_shadowing(&self, name: &str) -> bool {
        self.parent.as_ref().is_some_and(|p| p.is_defined(name))
    }
}

/// Check if a name is a built-in function
///
/// Returns true if the name is a Ruchy standard library function or built-in.
///
/// # Examples
///
/// ```
/// use ruchy::quality::linter::is_builtin;
///
/// assert!(is_builtin("println"));
/// assert!(is_builtin("fs_read"));
/// assert!(is_builtin("range"));
/// assert!(!is_builtin("my_custom_function"));
/// ```
pub fn is_builtin(name: &str) -> bool {
    matches!(
        name,
        // Output functions
        "println" | "print" | "eprintln" | "eprint" | "dbg" |
        // File system functions
        "fs_read" | "fs_write" | "fs_exists" | "fs_remove" | "fs_metadata" |
        "fs_create_dir" | "fs_read_dir" | "fs_copy" | "fs_rename" |
        // Environment functions
        "env_var" | "env_args" | "env_current_dir" | "env_set_var" |
        // HTTP functions
        "http_get" | "http_post" | "http_put" | "http_delete" |
        // JSON functions
        "json_parse" | "json_stringify" |
        // Time functions
        "time_now" | "time_sleep" | "time_duration" |
        // Path functions
        "path_join" | "path_extension" | "path_filename" | "path_parent" |
        // Collection functions
        "range" | "HashMap" | "HashSet" |
        // Math functions
        "abs" | "sqrt" | "pow" | "sin" | "cos" | "tan" | "floor" | "ceil" | "round" |
        "min" | "max" | "exp" | "ln" | "log10" | "log2" |
        // Process functions
        "exit" | "panic" | "assert" | "assert_eq" | "assert_ne" |
        // String functions (if any are global)
        "format" |
        // Regex functions
        "regex_new" | "regex_is_match" | "regex_find" | "regex_replace" |
        // Logging functions
        "log_info" | "log_warn" | "log_error" | "log_debug" | "log_trace" |
        // DataFrame functions
        "col" | "lit" | "DataFrame"
    )
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
    /// use ruchy::quality::linter::Linter;
    ///
    /// let instance = Linter::new();
    /// // Verify behavior
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
    /// use ruchy::quality::linter::Linter;
    ///
    /// let mut instance = Linter::new();
    /// let result = instance.set_rules();
    /// // Verify behavior
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
    /// ```ignore
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
    /// use ruchy::quality::linter::Linter;
    ///
    /// let mut instance = Linter::new();
    /// let result = instance.lint();
    /// // Verify behavior
    /// ```
    pub fn lint(&self, ast: &Expr, _source: &str) -> Result<Vec<LintIssue>> {
        let mut issues = Vec::new();
        let mut scope = Scope::new();

        // LINTER-086: Two-pass analysis for forward reference resolution (GitHub Issue #69)
        // Pass 1: Build symbol table (collect all function definitions)
        Self::collect_definitions(ast, &mut scope);

        // Pass 2: Analyze the AST with variable tracking (now with complete symbol table)
        self.analyze_expr(ast, &mut scope, &mut issues);

        // Check for unused variables
        self.check_unused_in_scope(&scope, &mut issues);
        // Check complexity
        if self
            .rules
            .iter()
            .any(|r| matches!(r, LintRule::ComplexityLimit))
            && Self::calculate_complexity(ast) > self.max_complexity
        {
            issues.push(LintIssue {
                line: 1,
                column: 1,
                severity: if self.strict_mode { "error" } else { "warning" }.to_string(),
                rule: "complexity".to_string(),
                message: format!(
                    "Function complexity exceeds limit of {}",
                    self.max_complexity
                ),
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

    /// Helper: Create shadowing `LintIssue` (CERTEZA-001: Reduce duplication)
    /// Complexity: 1 (within Toyota Way limits)
    #[inline]
    fn create_shadowing_issue(name: &str) -> LintIssue {
        LintIssue {
            line: 3, // Simplified line tracking
            column: 1,
            severity: "warning".to_string(),
            rule: "shadowing".to_string(),
            message: format!("variable shadowing: {name}"),
            suggestion: format!("Consider renaming variable '{name}'"),
            issue_type: "variable_shadowing".to_string(),
            name: name.to_string(),
        }
    }

    /// Helper: Create undefined variable `LintIssue` (CERTEZA-001: Reduce duplication)
    /// Complexity: 1 (within Toyota Way limits)
    #[inline]
    fn create_undefined_variable_issue(name: &str) -> LintIssue {
        LintIssue {
            line: 3,
            column: 1,
            severity: "error".to_string(),
            rule: "undefined".to_string(),
            message: format!("undefined variable: {name}"),
            suggestion: format!("Define '{name}' before using it"),
            issue_type: "undefined_variable".to_string(),
            name: name.to_string(),
        }
    }

    /// Helper: Create unused variable/parameter/binding `LintIssue` (CERTEZA-001: Reduce duplication)
    /// Complexity: 3 (within Toyota Way limits)
    #[inline]
    fn create_unused_issue(name: &str, var_type: VarType, defined_at: (usize, usize)) -> LintIssue {
        let (rule_type, message_prefix, suggestion_suffix) = match var_type {
            VarType::Local => ("unused_variable", "unused variable", "variable"),
            VarType::Parameter => ("unused_parameter", "unused parameter", "parameter"),
            VarType::LoopVariable => (
                "unused_loop_variable",
                "unused loop variable",
                "loop variable",
            ),
            VarType::MatchBinding => (
                "unused_match_binding",
                "unused match binding",
                "match binding",
            ),
            VarType::Function => ("unused_function", "unused function", "function"),
            VarType::TypeName => ("unused_type", "unused type", "type"),
        };

        LintIssue {
            line: defined_at.0,
            column: defined_at.1,
            severity: "warning".to_string(),
            rule: rule_type.to_string(),
            message: format!("{message_prefix}: {name}"),
            suggestion: format!("Remove unused {suggestion_suffix}"),
            issue_type: rule_type.to_string(),
            name: name.to_string(),
        }
    }

    /// LINTER-086: Pass 1 - Collect all function definitions for forward reference resolution
    /// Complexity: 4 (≤10 target)
    fn collect_definitions(expr: &Expr, scope: &mut Scope) {
        match &expr.kind {
            ExprKind::Function { name, .. } => {
                // Define function in symbol table (Pass 1)
                scope.define(name.clone(), 1, 1, VarType::Function);
            }
            ExprKind::Block(exprs) => {
                // Recursively collect definitions from block
                for expr in exprs {
                    Self::collect_definitions(expr, scope);
                }
            }
            ExprKind::Let { body, .. } => {
                // Recursively collect from let body
                Self::collect_definitions(body, scope);
            }
            _ => {
                // No definitions to collect for other expression types
            }
        }
    }
    fn analyze_expr(&self, expr: &Expr, scope: &mut Scope, issues: &mut Vec<LintIssue>) {
        match &expr.kind {
            ExprKind::Let {
                name, value, body, ..
            } => {
                // Analyze the value first (with current scope)
                self.analyze_expr(value, scope, issues);

                // Check if this is a top-level let (body is Unit) or expression-level let
                let is_top_level = matches!(body.kind, ExprKind::Literal(Literal::Unit));

                if is_top_level {
                    // Top-level let: Define variable in current scope (for use in subsequent statements)
                    // Check for shadowing before defining
                    if self
                        .rules
                        .iter()
                        .any(|r| matches!(r, LintRule::VariableShadowing))
                        && scope.is_shadowing(name)
                    {
                        issues.push(Self::create_shadowing_issue(name));
                    }
                    // Define in current scope for visibility in subsequent block statements
                    scope.define(name.clone(), 2, 1, VarType::Local);
                    // Analyze body (even if Unit) to maintain consistency
                    self.analyze_expr(body, scope, issues);
                } else {
                    // Expression-level let: Create new scope for the let binding body
                    let mut let_scope = Scope::with_parent(scope.clone());
                    // Check for shadowing before defining
                    if self
                        .rules
                        .iter()
                        .any(|r| matches!(r, LintRule::VariableShadowing))
                        && let_scope.is_shadowing(name)
                    {
                        issues.push(Self::create_shadowing_issue(name));
                    }
                    // Define the variable in the new scope
                    let_scope.define(name.clone(), 2, 1, VarType::Local);
                    // Analyze the body with the new scope
                    self.analyze_expr(body, &mut let_scope, issues);
                    // LINT-008 FIX: Propagate "used" status from cloned parent back to original scope
                    // The let_scope's parent is a clone of the original scope, so we need to sync the "used" flags
                    if let Some(parent_scope) = &let_scope.parent {
                        for (var_name, parent_var_info) in &parent_scope.variables {
                            if parent_var_info.used {
                                scope.mark_used(var_name);
                            }
                        }
                    }
                    // Check for unused variables in the let scope
                    self.check_unused_in_scope(&let_scope, issues);
                }
            }
            ExprKind::Identifier(name) => {
                // Skip built-in functions - they're always available
                if is_builtin(name) {
                    return;
                }
                // Mark as used if defined, otherwise report as undefined
                if !scope.mark_used(name)
                    && self
                        .rules
                        .iter()
                        .any(|r| matches!(r, LintRule::UndefinedVariable))
                {
                    issues.push(Self::create_undefined_variable_issue(name));
                }
            }
            ExprKind::Function {
                name, params, body, ..
            } => {
                // Define the function name in the current scope (as Function, not Local)
                scope.define(name.clone(), 1, 1, VarType::Function);
                // Create new scope for function body
                let mut func_scope = Scope::with_parent(scope.clone());
                // Add parameters to scope with correct type
                for param in params {
                    Self::extract_param_bindings(&param.pattern, &mut func_scope);
                }
                // Analyze function body
                self.analyze_expr(body, &mut func_scope, issues);
                // Check for unused variables in function body (but not parameters for now)
                // Parameters might be part of public API
                for (name, info) in &func_scope.variables {
                    if !info.used && matches!(info.var_type, VarType::Local) {
                        issues.push(Self::create_unused_issue(
                            name,
                            info.var_type.clone(),
                            info.defined_at,
                        ));
                    }
                }
            }
            ExprKind::For {
                label: None,
                var,
                pattern,
                iter,
                body,
                ..
            } => {
                // Create new scope for loop
                let mut loop_scope = Scope::with_parent(scope.clone());
                // Add loop variable to scope
                if let Some(pat) = pattern {
                    Self::extract_loop_bindings(pat, &mut loop_scope);
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
                    Self::extract_pattern_bindings(&arm.pattern, &mut branch_scope);
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
            ExprKind::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
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
                    Self::extract_param_bindings(&param.pattern, &mut lambda_scope);
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
            ExprKind::While {
                condition, body, ..
            } => {
                self.analyze_expr(condition, scope, issues);
                self.analyze_expr(body, scope, issues);
            }
            ExprKind::Assign { target, value, .. } => {
                self.analyze_expr(target, scope, issues);
                self.analyze_expr(value, scope, issues);
            }
            ExprKind::MacroInvocation { args, .. } => {
                // LINT-008: Visit macro arguments to mark variables as used (Issue #8)
                // format!("{}", name) should mark 'name' as used
                for arg in args {
                    self.analyze_expr(arg, scope, issues);
                }
            }
            ExprKind::Enum { name, .. } => {
                // Issue #107 FIX: Register enum type name in scope
                // This prevents "undefined variable" false positives for enum types
                scope.define(name.clone(), 1, 1, VarType::TypeName);
            }
            ExprKind::Struct { name, .. } => {
                // Issue #107 FIX: Register struct type name in scope
                // This prevents "undefined variable" false positives for struct types
                scope.define(name.clone(), 1, 1, VarType::TypeName);
            }
            _ => {
                // Handle other expression types as needed
            }
        }
    }
    fn extract_loop_bindings(pattern: &Pattern, scope: &mut Scope) {
        match pattern {
            Pattern::Identifier(name) => {
                // Check if it's a special identifier like _
                if name != "_" {
                    scope.define(name.clone(), 2, 1, VarType::LoopVariable);
                }
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    Self::extract_loop_bindings(p, scope);
                }
            }
            Pattern::Struct { fields, .. } => {
                for field in fields {
                    if let Some(pattern) = &field.pattern {
                        Self::extract_loop_bindings(pattern, scope);
                    } else {
                        // Shorthand: { x } means { x: x }, bind the name
                        scope.define(field.name.clone(), 2, 1, VarType::LoopVariable);
                    }
                }
            }
            Pattern::List(patterns) => {
                for p in patterns {
                    Self::extract_loop_bindings(p, scope);
                }
            }
            _ => {}
        }
    }
    fn extract_param_bindings(pattern: &Pattern, scope: &mut Scope) {
        match pattern {
            Pattern::Identifier(name) => {
                // Check if it's a special identifier like _
                if name != "_" {
                    scope.define(name.clone(), 1, 1, VarType::Parameter);
                }
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    Self::extract_param_bindings(p, scope);
                }
            }
            Pattern::Struct { fields, .. } => {
                for field in fields {
                    if let Some(pattern) = &field.pattern {
                        Self::extract_param_bindings(pattern, scope);
                    } else {
                        // Shorthand: { x } means { x: x }, bind the name
                        scope.define(field.name.clone(), 1, 1, VarType::Parameter);
                    }
                }
            }
            Pattern::List(patterns) => {
                for p in patterns {
                    Self::extract_param_bindings(p, scope);
                }
            }
            _ => {}
        }
    }
    fn extract_pattern_bindings(pattern: &Pattern, scope: &mut Scope) {
        match pattern {
            Pattern::Identifier(name) => {
                // Check if it's a special identifier like _
                if name != "_" {
                    scope.define(name.clone(), 3, 1, VarType::MatchBinding);
                }
            }
            Pattern::Tuple(patterns) => {
                for p in patterns {
                    Self::extract_pattern_bindings(p, scope);
                }
            }
            Pattern::Struct { fields, .. } => {
                for field in fields {
                    if let Some(pattern) = &field.pattern {
                        Self::extract_pattern_bindings(pattern, scope);
                    } else {
                        // Shorthand: { x } means { x: x }, bind the name
                        scope.define(field.name.clone(), 3, 1, VarType::MatchBinding);
                    }
                }
            }
            Pattern::List(patterns) => {
                for p in patterns {
                    Self::extract_pattern_bindings(p, scope);
                }
            }
            Pattern::Some(inner) | Pattern::Ok(inner) | Pattern::Err(inner) => {
                Self::extract_pattern_bindings(inner, scope);
            }
            _ => {}
        }
    }
    fn check_unused_in_scope(&self, scope: &Scope, issues: &mut Vec<LintIssue>) {
        for (name, info) in &scope.variables {
            if !info.used {
                // Check if rule is enabled for this variable type
                let should_check = match info.var_type {
                    VarType::Local => self
                        .rules
                        .iter()
                        .any(|r| matches!(r, LintRule::UnusedVariable)),
                    VarType::Parameter => self
                        .rules
                        .iter()
                        .any(|r| matches!(r, LintRule::UnusedParameter)),
                    VarType::Function => false, // QUALITY-015: Functions not checked for unused
                    VarType::LoopVariable => self
                        .rules
                        .iter()
                        .any(|r| matches!(r, LintRule::UnusedLoopVariable)),
                    VarType::MatchBinding => self
                        .rules
                        .iter()
                        .any(|r| matches!(r, LintRule::UnusedMatchBinding)),
                    VarType::TypeName => false, // Issue #107: Type names not checked
                };

                if should_check {
                    issues.push(Self::create_unused_issue(
                        name,
                        info.var_type.clone(),
                        info.defined_at,
                    ));
                }
            }
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::quality::linter::Linter;
    ///
    /// let mut instance = Linter::new();
    /// let result = instance.auto_fix();
    /// // Verify behavior
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
    fn calculate_complexity(expr: &Expr) -> usize {
        match &expr.kind {
            ExprKind::If {
                condition: _,
                then_branch,
                else_branch,
                ..
            } => {
                1 + Self::calculate_complexity(then_branch)
                    + else_branch
                        .as_ref()
                        .map_or(0, |e| Self::calculate_complexity(e))
            }
            ExprKind::Match { .. } => 2,
            ExprKind::While { .. } | ExprKind::For { .. } => 2,
            ExprKind::Block(exprs) => exprs.iter().map(Self::calculate_complexity).sum(),
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
#[allow(unused_variables)]
#[path = "linter_core_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "linter_prop_tests.rs"]
mod property_tests_linter;

#[cfg(test)]
#[path = "linter_sprint44_tests.rs"]
mod sprint_44_tests;
