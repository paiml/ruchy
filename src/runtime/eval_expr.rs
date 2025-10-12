//! Expression evaluation dispatch module
//!
//! This module handles the main expression evaluation dispatch, including
//! control flow expressions and data structure construction.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::frontend::ast::{Expr, ExprKind, Literal};
use crate::runtime::{InterpreterError, Value};
use std::sync::Arc;

/// Main expression evaluation dispatcher
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
pub fn eval_expr_kind<F>(
    expr_kind: &ExprKind,
    _eval_expr: F,
    mut lookup_variable: impl FnMut(&str) -> Result<Value, InterpreterError>,
    mut eval_function: impl FnMut(
        &Option<String>,
        &[crate::frontend::Param],
        &Expr,
    ) -> Result<Value, InterpreterError>,
    mut eval_lambda: impl FnMut(&[crate::frontend::Param], &Expr) -> Result<Value, InterpreterError>,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    match expr_kind {
        // Basic expressions - inlined for performance
        ExprKind::Literal(lit) => Ok(eval_literal(lit)),
        ExprKind::Identifier(name) => lookup_variable(name),

        // Functions and lambdas
        ExprKind::Function {
            name, params, body, ..
        } => eval_function(&Some(name.clone()), params, body),
        ExprKind::Lambda { params, body } => eval_lambda(params, body),

        // Delegated to caller for complex operations
        _ => Err(InterpreterError::RuntimeError(
            "Expression requires interpreter context".to_string(),
        )),
    }
}

/// Evaluate a literal value
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
pub fn eval_literal(lit: &Literal) -> Value {
    match lit {
        Literal::Integer(i, _) => Value::Integer(*i),
        Literal::Float(f) => Value::Float(*f),
        Literal::String(s) => Value::from_string(s.clone()),
        Literal::Bool(b) => Value::Bool(*b),
        Literal::Char(c) => Value::from_string(c.to_string()),
        Literal::Byte(b) => Value::Byte(*b),
        Literal::Unit => Value::Nil,
        Literal::Null => Value::Nil,
    }
}

/// Check if an expression is a control flow expression
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
pub fn is_control_flow_expr(expr_kind: &ExprKind) -> bool {
    matches!(
        expr_kind,
        ExprKind::If { .. }
            | ExprKind::Ternary { .. }
            | ExprKind::Let { .. }
            | ExprKind::For { .. }
            | ExprKind::While { .. }
            | ExprKind::Loop { .. }
            | ExprKind::Match { .. }
            | ExprKind::Break { .. }
            | ExprKind::Continue { .. }
            | ExprKind::Return { .. }
            | ExprKind::TryCatch { .. }
            | ExprKind::Throw { .. }
    )
}

/// Check if an expression is a data structure expression
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
pub fn is_data_structure_expr(expr_kind: &ExprKind) -> bool {
    matches!(
        expr_kind,
        ExprKind::List(_)
            | ExprKind::Block(_)
            | ExprKind::Tuple(_)
            | ExprKind::Range { .. }
            | ExprKind::ArrayInit { .. }
            | ExprKind::DataFrame { .. }
    )
}

/// Check if an expression is an assignment expression
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
pub fn is_assignment_expr(expr_kind: &ExprKind) -> bool {
    matches!(
        expr_kind,
        ExprKind::Assign { .. } | ExprKind::CompoundAssign { .. }
    )
}

/// Evaluate an if expression
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_if_expr<F>(
    condition: &Expr,
    then_branch: &Expr,
    else_branch: Option<&Expr>,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let condition_val = eval_expr(condition)?;
    if condition_val.is_truthy() {
        eval_expr(then_branch)
    } else if let Some(else_expr) = else_branch {
        eval_expr(else_expr)
    } else {
        Ok(Value::Nil)
    }
}

/// Evaluate a let expression
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_let_expr<F>(
    name: &str,
    value: &Expr,
    body: &Expr,
    mut eval_expr: F,
    mut env_set: impl FnMut(String, Value),
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let val = eval_expr(value)?;
    env_set(name.to_string(), val.clone());

    // If body is unit (empty), return the value like REPL does
    // This makes `let x = 42` return 42 instead of nil
    match &body.kind {
        ExprKind::Literal(Literal::Unit) => Ok(val),
        _ => eval_expr(body),
    }
}

/// Evaluate a return expression
///
/// Returns an `InterpreterError::Return` variant which is caught by function call evaluation.
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_return_expr<F>(
    value: Option<&Expr>,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    if let Some(expr) = value {
        let val = eval_expr(expr)?;
        Err(InterpreterError::Return(val))
    } else {
        Err(InterpreterError::Return(Value::Nil))
    }
}

/// Evaluate a list expression
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_list_expr<F>(elements: &[Expr], mut eval_expr: F) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut values = Vec::new();
    for elem in elements {
        values.push(eval_expr(elem)?);
    }
    Ok(Value::from_array(values))
}

/// Evaluate an array initialization expression [value; size]
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_array_init_expr<F>(
    value_expr: &Expr,
    size_expr: &Expr,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let value = eval_expr(value_expr)?;
    let size_val = eval_expr(size_expr)?;

    let size = match size_val {
        Value::Integer(n) => n as usize,
        _ => {
            return Err(InterpreterError::RuntimeError(
                "Array size must be an integer".to_string(),
            ))
        }
    };

    let mut values = Vec::with_capacity(size);
    for _ in 0..size {
        values.push(value.clone());
    }

    Ok(Value::from_array(values))
}

/// Evaluate a block expression
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_block_expr<F>(statements: &[Expr], mut eval_expr: F) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut result = Value::Nil;
    for stmt in statements {
        result = eval_expr(stmt)?;
    }
    Ok(result)
}

/// Evaluate a tuple expression
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
pub fn eval_tuple_expr<F>(elements: &[Expr], mut eval_expr: F) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let mut values = Vec::new();
    for elem in elements {
        values.push(eval_expr(elem)?);
    }
    Ok(Value::Tuple(Arc::from(values.as_slice())))
}

/// Evaluate a range expression
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
pub fn eval_range_expr<F>(
    start: &Expr,
    end: &Expr,
    inclusive: bool,
    mut eval_expr: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Expr) -> Result<Value, InterpreterError>,
{
    let start_val = eval_expr(start)?;
    let end_val = eval_expr(end)?;

    match (&start_val, &end_val) {
        (Value::Integer(_), Value::Integer(_)) => Ok(Value::Range {
            start: Box::new(start_val),
            end: Box::new(end_val),
            inclusive,
        }),
        _ => Err(InterpreterError::TypeError(
            "Range bounds must be integers".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::ast::Span;

    fn make_literal_expr(lit: Literal) -> Expr {
        Expr::new(ExprKind::Literal(lit), Span::default())
    }

    #[test]
    fn test_eval_literal() {
        assert_eq!(
            eval_literal(&Literal::Integer(42, None)),
            Value::Integer(42)
        );
        assert_eq!(eval_literal(&Literal::Float(3.14)), Value::Float(3.14));
        assert_eq!(eval_literal(&Literal::Bool(true)), Value::Bool(true));
        assert_eq!(eval_literal(&Literal::Unit), Value::Nil);
        assert_eq!(eval_literal(&Literal::Null), Value::Nil);

        let s = eval_literal(&Literal::String("test".to_string()));
        match s {
            Value::String(s) => assert_eq!(s.as_ref(), "test"),
            _ => panic!("Expected string value"),
        }
    }

    #[test]
    fn test_is_control_flow() {
        let if_expr = ExprKind::If {
            condition: Box::new(make_literal_expr(Literal::Bool(true))),
            then_branch: Box::new(make_literal_expr(Literal::Integer(1, None))),
            else_branch: None,
        };
        assert!(is_control_flow_expr(&if_expr));

        let literal_expr = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!is_control_flow_expr(&literal_expr));
    }

    #[test]
    fn test_is_data_structure() {
        let list_expr = ExprKind::List(vec![]);
        assert!(is_data_structure_expr(&list_expr));

        let tuple_expr = ExprKind::Tuple(vec![]);
        assert!(is_data_structure_expr(&tuple_expr));

        let literal_expr = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!is_data_structure_expr(&literal_expr));
    }

    #[test]
    fn test_eval_if_expr() {
        let condition = make_literal_expr(Literal::Bool(true));
        let then_branch = make_literal_expr(Literal::Integer(1, None));
        let else_branch = make_literal_expr(Literal::Integer(2, None));

        let result = eval_if_expr(
            &condition,
            &then_branch,
            Some(&else_branch),
            |expr| match &expr.kind {
                ExprKind::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
                ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
                _ => Ok(Value::Nil),
            },
        )
        .unwrap();

        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_eval_list_expr() {
        let elements = vec![
            make_literal_expr(Literal::Integer(1, None)),
            make_literal_expr(Literal::Integer(2, None)),
            make_literal_expr(Literal::Integer(3, None)),
        ];

        let result = eval_list_expr(&elements, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Integer(1));
                assert_eq!(arr[1], Value::Integer(2));
                assert_eq!(arr[2], Value::Integer(3));
            }
            _ => panic!("Expected array value"),
        }
    }
}
