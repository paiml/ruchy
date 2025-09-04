//! Expression evaluation module for REPL
//! Handles all expression evaluation with low complexity per function

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::time::Instant;

use crate::frontend::ast::{
    BinaryOp, Expr, ExprKind, Literal, Pattern, UnaryOp, MatchArm,
};
use crate::runtime::repl::Value;

/// Configuration for evaluation limits
pub struct EvaluationConfig {
    pub max_depth: usize,
    pub max_iterations: usize,
    pub timeout_seconds: u64,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            max_depth: 1000,
            max_iterations: 100_000,
            timeout_seconds: 30,
        }
    }
}

/// Context for expression evaluation
pub struct EvaluationContext<'a> {
    pub bindings: &'a mut dyn BindingProvider,
    pub functions: &'a HashMap<String, (Vec<String>, Box<Expr>)>,
    pub config: &'a EvaluationConfig,
}

/// Trait for providing variable bindings
pub trait BindingProvider {
    fn get_binding(&self, name: &str) -> Option<Value>;
    fn set_binding(&mut self, name: String, value: Value, is_mutable: bool) -> Result<()>;
    fn push_scope(&mut self);
    fn pop_scope(&mut self);
}

/// Main evaluation entry point (complexity: 10)
pub fn evaluate_expression(
    expr: &Expr,
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    // Check resource bounds
    check_resource_bounds(deadline, depth, context.config)?;
    
    // Dispatch to appropriate evaluator
    match &expr.kind {
        // Literals
        ExprKind::Literal(lit) => evaluate_literal(lit),
        
        // Variables
        ExprKind::Identifier(name) => evaluate_identifier(name, context),
        
        // Binary operations
        ExprKind::Binary { left, op, right } => {
            evaluate_binary_operation(left, *op, right, context, deadline, depth)
        }
        
        // Unary operations
        ExprKind::Unary { op, operand } => {
            evaluate_unary_operation(*op, operand, context, deadline, depth)
        }
        
        // Control flow
        ExprKind::If { condition, then_branch, else_branch } => {
            evaluate_if_expression(condition, then_branch, else_branch.as_deref(), context, deadline, depth)
        }
        
        // Loops
        ExprKind::For { var, iter, body, .. } => {
            evaluate_for_loop(var, iter, body, context, deadline, depth)
        }
        
        ExprKind::While { condition, body } => {
            evaluate_while_loop(condition, body, context, deadline, depth)
        }
        
        // Data structures
        ExprKind::List(elements) => {
            evaluate_list(elements, context, deadline, depth)
        }
        
        ExprKind::Tuple(elements) => {
            evaluate_tuple(elements, context, deadline, depth)
        }
        
        // Function calls
        ExprKind::Call { callee, args } => {
            evaluate_call(callee, args, context, deadline, depth)
        }
        
        // Method calls
        ExprKind::MethodCall { object, method, args } => {
            evaluate_method_call(object, method, args, context, deadline, depth)
        }
        
        // Pattern matching
        ExprKind::Match { expr: match_expr, arms } => {
            evaluate_match(match_expr, arms, context, deadline, depth)
        }
        
        _ => bail!("Unsupported expression type: {:?}", expr.kind),
    }
}

/// Check resource bounds (complexity: 3)
fn check_resource_bounds(deadline: Instant, depth: usize, config: &EvaluationConfig) -> Result<()> {
    if Instant::now() > deadline {
        bail!("Evaluation timeout exceeded");
    }
    if depth > config.max_depth {
        bail!("Maximum recursion depth {} exceeded", config.max_depth);
    }
    Ok(())
}

/// Evaluate literal (complexity: 5)
fn evaluate_literal(lit: &Literal) -> Result<Value> {
    match lit {
        Literal::Nil => Ok(Value::Nil),
        Literal::Bool(b) => Ok(Value::Bool(*b)),
        Literal::Int(n) => Ok(Value::Int(*n)),
        Literal::Float(f) => Ok(Value::Float(*f)),
        Literal::String(s) => Ok(Value::String(s.clone())),
        Literal::Char(c) => Ok(Value::Char(*c)),
    }
}

/// Evaluate identifier (complexity: 2)
fn evaluate_identifier(name: &str, context: &EvaluationContext) -> Result<Value> {
    context.bindings.get_binding(name)
        .ok_or_else(|| anyhow::anyhow!("Undefined variable: {}", name))
}

/// Evaluate binary operation (complexity: 8)
fn evaluate_binary_operation(
    left: &Expr,
    op: BinaryOp,
    right: &Expr,
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    // Handle short-circuit operators
    match op {
        BinaryOp::And => {
            let lhs = evaluate_expression(left, context, deadline, depth + 1)?;
            if !is_truthy(&lhs) {
                return Ok(lhs);
            }
            evaluate_expression(right, context, deadline, depth + 1)
        }
        BinaryOp::Or => {
            let lhs = evaluate_expression(left, context, deadline, depth + 1)?;
            if is_truthy(&lhs) {
                return Ok(lhs);
            }
            evaluate_expression(right, context, deadline, depth + 1)
        }
        BinaryOp::NullCoalesce => {
            let lhs = evaluate_expression(left, context, deadline, depth + 1)?;
            if !matches!(lhs, Value::Nil) {
                return Ok(lhs);
            }
            evaluate_expression(right, context, deadline, depth + 1)
        }
        _ => {
            // Evaluate both operands for arithmetic/comparison
            let lhs = evaluate_expression(left, context, deadline, depth + 1)?;
            let rhs = evaluate_expression(right, context, deadline, depth + 1)?;
            apply_binary_operator(&lhs, op, &rhs)
        }
    }
}

/// Evaluate unary operation (complexity: 4)
fn evaluate_unary_operation(
    op: UnaryOp,
    operand: &Expr,
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    let val = evaluate_expression(operand, context, deadline, depth + 1)?;
    apply_unary_operator(op, &val)
}

/// Apply binary operator to values (complexity: 10)
fn apply_binary_operator(lhs: &Value, op: BinaryOp, rhs: &Value) -> Result<Value> {
    use BinaryOp::*;
    
    match op {
        // Arithmetic
        Add => add_values(lhs, rhs),
        Sub => subtract_values(lhs, rhs),
        Mul => multiply_values(lhs, rhs),
        Div => divide_values(lhs, rhs),
        Mod => modulo_values(lhs, rhs),
        Pow => power_values(lhs, rhs),
        
        // Comparison
        Equal => Ok(Value::Bool(values_equal(lhs, rhs))),
        NotEqual => Ok(Value::Bool(!values_equal(lhs, rhs))),
        Less => compare_less(lhs, rhs),
        LessEqual => compare_less_equal(lhs, rhs),
        Greater => compare_greater(lhs, rhs),
        GreaterEqual => compare_greater_equal(lhs, rhs),
        
        // Bitwise
        BitwiseAnd => bitwise_and(lhs, rhs),
        BitwiseOr => bitwise_or(lhs, rhs),
        BitwiseXor => bitwise_xor(lhs, rhs),
        LeftShift => left_shift(lhs, rhs),
        RightShift => right_shift(lhs, rhs),
        
        _ => bail!("Unsupported binary operator: {:?}", op),
    }
}

/// Apply unary operator to value (complexity: 5)
fn apply_unary_operator(op: UnaryOp, val: &Value) -> Result<Value> {
    match op {
        UnaryOp::Not => match val {
            Value::Bool(b) => Ok(Value::Bool(!b)),
            _ => bail!("Cannot apply '!' to non-boolean value"),
        },
        UnaryOp::Neg => match val {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            _ => bail!("Cannot negate non-numeric value"),
        },
        UnaryOp::BitwiseNot => match val {
            Value::Int(n) => Ok(Value::Int(!n)),
            _ => bail!("Bitwise NOT requires integer"),
        },
        _ => bail!("Unsupported unary operator: {:?}", op),
    }
}

// ==================== Arithmetic Operations ====================

fn add_values(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (Value::Int(a), Value::Float(b)) | (Value::Float(b), Value::Int(a)) => {
            Ok(Value::Float(*a as f64 + b))
        }
        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
        (Value::List(a), Value::List(b)) => {
            let mut result = a.clone();
            result.extend(b.clone());
            Ok(Value::List(result))
        }
        _ => bail!("Cannot add {:?} and {:?}", lhs, rhs),
    }
}

fn subtract_values(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float(*a as f64 - b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - *b as f64)),
        _ => bail!("Cannot subtract {:?} from {:?}", rhs, lhs),
    }
}

fn multiply_values(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (Value::Int(a), Value::Float(b)) | (Value::Float(b), Value::Int(a)) => {
            Ok(Value::Float(*a as f64 * b))
        }
        (Value::String(s), Value::Int(n)) | (Value::Int(n), Value::String(s)) => {
            if *n < 0 {
                bail!("Cannot repeat string negative times");
            }
            Ok(Value::String(s.repeat(*n as usize)))
        }
        _ => bail!("Cannot multiply {:?} and {:?}", lhs, rhs),
    }
}

fn divide_values(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            if *b == 0 {
                bail!("Division by zero");
            }
            Ok(Value::Int(a / b))
        }
        (Value::Float(a), Value::Float(b)) => {
            if *b == 0.0 {
                bail!("Division by zero");
            }
            Ok(Value::Float(a / b))
        }
        (Value::Int(a), Value::Float(b)) => {
            if *b == 0.0 {
                bail!("Division by zero");
            }
            Ok(Value::Float(*a as f64 / b))
        }
        (Value::Float(a), Value::Int(b)) => {
            if *b == 0 {
                bail!("Division by zero");
            }
            Ok(Value::Float(a / *b as f64))
        }
        _ => bail!("Cannot divide {:?} by {:?}", lhs, rhs),
    }
}

fn modulo_values(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            if *b == 0 {
                bail!("Modulo by zero");
            }
            Ok(Value::Int(a % b))
        }
        _ => bail!("Modulo requires integers"),
    }
}

fn power_values(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            if *b < 0 {
                bail!("Negative exponent requires float");
            }
            Ok(Value::Int(a.pow(*b as u32)))
        }
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.powf(*b))),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Float((*a as f64).powf(*b))),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a.powi(*b as i32))),
        _ => bail!("Cannot raise {:?} to power {:?}", lhs, rhs),
    }
}

// ==================== Comparison Operations ====================

fn values_equal(lhs: &Value, rhs: &Value) -> bool {
    match (lhs, rhs) {
        (Value::Nil, Value::Nil) => true,
        (Value::Bool(a), Value::Bool(b)) => a == b,
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::List(a), Value::List(b)) => {
            a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| values_equal(x, y))
        }
        _ => false,
    }
}

fn compare_less(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) < *b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a < (*b as f64))),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a < b)),
        _ => bail!("Cannot compare {:?} < {:?}", lhs, rhs),
    }
}

fn compare_less_equal(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) <= *b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a <= (*b as f64))),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a <= b)),
        _ => bail!("Cannot compare {:?} <= {:?}", lhs, rhs),
    }
}

fn compare_greater(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) > *b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a > (*b as f64))),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a > b)),
        _ => bail!("Cannot compare {:?} > {:?}", lhs, rhs),
    }
}

fn compare_greater_equal(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
        (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((*a as f64) >= *b)),
        (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(*a >= (*b as f64))),
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a >= b)),
        _ => bail!("Cannot compare {:?} >= {:?}", lhs, rhs),
    }
}

// ==================== Bitwise Operations ====================

fn bitwise_and(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a & b)),
        _ => bail!("Bitwise AND requires integers"),
    }
}

fn bitwise_or(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a | b)),
        _ => bail!("Bitwise OR requires integers"),
    }
}

fn bitwise_xor(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a ^ b)),
        _ => bail!("Bitwise XOR requires integers"),
    }
}

fn left_shift(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            if *b < 0 || *b >= 64 {
                bail!("Shift amount must be between 0 and 63");
            }
            Ok(Value::Int(a << b))
        }
        _ => bail!("Left shift requires integers"),
    }
}

fn right_shift(lhs: &Value, rhs: &Value) -> Result<Value> {
    match (lhs, rhs) {
        (Value::Int(a), Value::Int(b)) => {
            if *b < 0 || *b >= 64 {
                bail!("Shift amount must be between 0 and 63");
            }
            Ok(Value::Int(a >> b))
        }
        _ => bail!("Right shift requires integers"),
    }
}

// ==================== Control Flow ====================

/// Evaluate if expression (complexity: 6)
fn evaluate_if_expression(
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    let cond_val = evaluate_expression(condition, context, deadline, depth + 1)?;
    
    if is_truthy(&cond_val) {
        evaluate_expression(then_branch, context, deadline, depth + 1)
    } else if let Some(else_expr) = else_branch {
        evaluate_expression(else_expr, context, deadline, depth + 1)
    } else {
        Ok(Value::Unit)
    }
}

/// Evaluate for loop (complexity: 10)
fn evaluate_for_loop(
    var: &str,
    iter: &Expr,
    body: &Expr,
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    let iterable = evaluate_expression(iter, context, deadline, depth + 1)?;
    let items = extract_iterable_items(iterable)?;
    
    let mut last_value = Value::Unit;
    let mut iterations = 0;
    
    for item in items {
        // Check iteration limit
        iterations += 1;
        if iterations > context.config.max_iterations {
            bail!("Maximum iteration limit {} exceeded", context.config.max_iterations);
        }
        
        // Check timeout
        check_resource_bounds(deadline, depth + 1, context.config)?;
        
        // Bind loop variable
        context.bindings.push_scope();
        context.bindings.set_binding(var.to_string(), item, false)?;
        
        // Execute body
        match evaluate_expression(body, context, deadline, depth + 1) {
            Ok(value) => last_value = value,
            Err(e) if e.to_string() == "break" => {
                context.bindings.pop_scope();
                break;
            }
            Err(e) if e.to_string() == "continue" => {
                context.bindings.pop_scope();
                continue;
            }
            Err(e) => {
                context.bindings.pop_scope();
                return Err(e);
            }
        }
        
        context.bindings.pop_scope();
    }
    
    Ok(last_value)
}

/// Evaluate while loop (complexity: 7)
fn evaluate_while_loop(
    condition: &Expr,
    body: &Expr,
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    let mut last_value = Value::Unit;
    let mut iterations = 0;
    
    loop {
        // Check iteration limit
        iterations += 1;
        if iterations > context.config.max_iterations {
            bail!("Maximum iteration limit {} exceeded", context.config.max_iterations);
        }
        
        // Check timeout
        check_resource_bounds(deadline, depth + 1, context.config)?;
        
        // Evaluate condition
        let cond_val = evaluate_expression(condition, context, deadline, depth + 1)?;
        if !is_truthy(&cond_val) {
            break;
        }
        
        // Execute body
        match evaluate_expression(body, context, deadline, depth + 1) {
            Ok(value) => last_value = value,
            Err(e) if e.to_string() == "break" => break,
            Err(e) if e.to_string() == "continue" => {},
            Err(e) => return Err(e),
        }
    }
    
    Ok(last_value)
}

// ==================== Data Structures ====================

/// Evaluate list literal (complexity: 5)
fn evaluate_list(
    elements: &[Expr],
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    let mut values = Vec::new();
    for elem in elements {
        values.push(evaluate_expression(elem, context, deadline, depth + 1)?);
    }
    Ok(Value::List(values))
}

/// Evaluate tuple literal (complexity: 5)
fn evaluate_tuple(
    elements: &[Expr],
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    let mut values = Vec::new();
    for elem in elements {
        values.push(evaluate_expression(elem, context, deadline, depth + 1)?);
    }
    Ok(Value::Tuple(values))
}

// ==================== Function Calls ====================

/// Evaluate function call (complexity: 10)
fn evaluate_call(
    callee: &Expr,
    args: &[Expr],
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    // Check for built-in functions
    if let ExprKind::Identifier(name) = &callee.kind {
        if let Some(result) = try_builtin_function(name, args, context, deadline, depth)? {
            return Ok(result);
        }
    }
    
    // Evaluate callee
    let func_val = evaluate_expression(callee, context, deadline, depth + 1)?;
    
    // Evaluate arguments
    let mut arg_values = Vec::new();
    for arg in args {
        arg_values.push(evaluate_expression(arg, context, deadline, depth + 1)?);
    }
    
    // Apply function
    apply_function(func_val, arg_values, context, deadline, depth)
}

/// Try to execute built-in function (complexity: 8)
fn try_builtin_function(
    name: &str,
    args: &[Expr],
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Option<Value>> {
    match name {
        "println" | "print" => {
            let mut output = Vec::new();
            for arg in args {
                let val = evaluate_expression(arg, context, deadline, depth + 1)?;
                output.push(format!("{}", val));
            }
            println!("{}", output.join(" "));
            Ok(Some(Value::Unit))
        }
        "len" | "length" => {
            if args.len() != 1 {
                bail!("len() expects 1 argument");
            }
            let val = evaluate_expression(&args[0], context, deadline, depth + 1)?;
            match val {
                Value::String(s) => Ok(Some(Value::Int(s.len() as i64))),
                Value::List(l) => Ok(Some(Value::Int(l.len() as i64))),
                _ => bail!("len() requires string or list"),
            }
        }
        _ => Ok(None),
    }
}

/// Apply function to arguments (complexity: 8)
fn apply_function(
    func: Value,
    args: Vec<Value>,
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    match func {
        Value::Function { params, body, .. } | Value::Lambda { params, body } => {
            if params.len() != args.len() {
                bail!("Function expects {} arguments, got {}", params.len(), args.len());
            }
            
            // Create new scope for function
            context.bindings.push_scope();
            
            // Bind parameters
            for (param, arg) in params.iter().zip(args) {
                context.bindings.set_binding(param.clone(), arg, false)?;
            }
            
            // Execute function body
            let result = evaluate_expression(&body, context, deadline, depth + 1);
            
            // Pop scope
            context.bindings.pop_scope();
            
            result
        }
        _ => bail!("Cannot call non-function value"),
    }
}

/// Evaluate method call (complexity: 8)
fn evaluate_method_call(
    object: &Expr,
    method: &str,
    args: &[Expr],
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    // Evaluate object
    let obj = evaluate_expression(object, context, deadline, depth + 1)?;
    
    // Evaluate arguments
    let mut arg_values = Vec::new();
    for arg in args {
        arg_values.push(evaluate_expression(arg, context, deadline, depth + 1)?);
    }
    
    // Dispatch to method handler (would call into methods module)
    bail!("Method call evaluation not yet implemented: {}.{}", obj, method)
}

// ==================== Pattern Matching ====================

/// Evaluate match expression (complexity: 10)
fn evaluate_match(
    expr: &Expr,
    arms: &[MatchArm],
    context: &mut EvaluationContext,
    deadline: Instant,
    depth: usize,
) -> Result<Value> {
    let value = evaluate_expression(expr, context, deadline, depth + 1)?;
    
    for arm in arms {
        if let Some(bindings) = match_pattern(&arm.pattern, &value) {
            // Create new scope for match arm
            context.bindings.push_scope();
            
            // Bind pattern variables
            for (name, val) in bindings {
                context.bindings.set_binding(name, val, false)?;
            }
            
            // Check guard if present
            let guard_passed = if let Some(guard) = &arm.guard {
                let guard_val = evaluate_expression(guard, context, deadline, depth + 1)?;
                is_truthy(&guard_val)
            } else {
                true
            };
            
            if guard_passed {
                // Execute arm body
                let result = evaluate_expression(&arm.body, context, deadline, depth + 1);
                context.bindings.pop_scope();
                return result;
            }
            
            context.bindings.pop_scope();
        }
    }
    
    bail!("No match arm matched the value")
}

/// Match a pattern against a value (complexity: 8)
fn match_pattern(pattern: &Pattern, value: &Value) -> Option<Vec<(String, Value)>> {
    match pattern {
        Pattern::Wildcard => Some(vec![]),
        Pattern::Literal(lit) => {
            let lit_val = evaluate_literal(lit).ok()?;
            if values_equal(&lit_val, value) {
                Some(vec![])
            } else {
                None
            }
        }
        Pattern::Identifier(name) => Some(vec![(name.clone(), value.clone())]),
        Pattern::List(patterns) => {
            if let Value::List(values) = value {
                if patterns.len() != values.len() {
                    return None;
                }
                let mut bindings = Vec::new();
                for (pat, val) in patterns.iter().zip(values) {
                    bindings.extend(match_pattern(pat, val)?);
                }
                Some(bindings)
            } else {
                None
            }
        }
        Pattern::Tuple(patterns) => {
            if let Value::Tuple(values) = value {
                if patterns.len() != values.len() {
                    return None;
                }
                let mut bindings = Vec::new();
                for (pat, val) in patterns.iter().zip(values) {
                    bindings.extend(match_pattern(pat, val)?);
                }
                Some(bindings)
            } else {
                None
            }
        }
        _ => None, // TODO: Implement other patterns
    }
}

// ==================== Utility Functions ====================

/// Check if a value is truthy (complexity: 3)
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::List(l) => !l.is_empty(),
        _ => true,
    }
}

/// Extract iterable items from a value (complexity: 5)
fn extract_iterable_items(value: Value) -> Result<Vec<Value>> {
    match value {
        Value::List(items) => Ok(items),
        Value::Tuple(items) => Ok(items),
        Value::Range { start, end, inclusive } => {
            let mut items = Vec::new();
            let actual_end = if inclusive { end + 1 } else { end };
            if actual_end - start > 10000 {
                bail!("Range too large (>10000 items)");
            }
            for i in start..actual_end {
                items.push(Value::Int(i));
            }
            Ok(items)
        }
        Value::String(s) => {
            Ok(s.chars().map(|c| Value::Char(c)).collect())
        }
        _ => bail!("Cannot iterate over {:?}", value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBindings {
        bindings: Vec<HashMap<String, Value>>,
    }

    impl TestBindings {
        fn new() -> Self {
            Self {
                bindings: vec![HashMap::new()],
            }
        }
    }

    impl BindingProvider for TestBindings {
        fn get_binding(&self, name: &str) -> Option<Value> {
            for scope in self.bindings.iter().rev() {
                if let Some(val) = scope.get(name) {
                    return Some(val.clone());
                }
            }
            None
        }

        fn set_binding(&mut self, name: String, value: Value, _is_mutable: bool) -> Result<()> {
            if let Some(scope) = self.bindings.last_mut() {
                scope.insert(name, value);
            }
            Ok(())
        }

        fn push_scope(&mut self) {
            self.bindings.push(HashMap::new());
        }

        fn pop_scope(&mut self) {
            self.bindings.pop();
        }
    }

    #[test]
    fn test_evaluate_literal() {
        assert_eq!(evaluate_literal(&Literal::Int(42)).unwrap(), Value::Int(42));
        assert_eq!(evaluate_literal(&Literal::Bool(true)).unwrap(), Value::Bool(true));
        assert_eq!(evaluate_literal(&Literal::String("hello".to_string())).unwrap(), 
                   Value::String("hello".to_string()));
    }

    #[test]
    fn test_is_truthy() {
        assert!(is_truthy(&Value::Bool(true)));
        assert!(!is_truthy(&Value::Bool(false)));
        assert!(is_truthy(&Value::Int(1)));
        assert!(!is_truthy(&Value::Int(0)));
        assert!(!is_truthy(&Value::Nil));
    }

    #[test]
    fn test_arithmetic_operations() {
        assert_eq!(add_values(&Value::Int(2), &Value::Int(3)).unwrap(), Value::Int(5));
        assert_eq!(multiply_values(&Value::Int(4), &Value::Int(5)).unwrap(), Value::Int(20));
        assert_eq!(divide_values(&Value::Int(10), &Value::Int(2)).unwrap(), Value::Int(5));
    }

    #[test]
    fn test_comparison_operations() {
        assert!(values_equal(&Value::Int(42), &Value::Int(42)));
        assert!(!values_equal(&Value::Int(42), &Value::Int(43)));
        assert_eq!(compare_less(&Value::Int(1), &Value::Int(2)).unwrap(), Value::Bool(true));
        assert_eq!(compare_greater(&Value::Int(5), &Value::Int(3)).unwrap(), Value::Bool(true));
    }
}