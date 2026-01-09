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
        Literal::Atom(s) => Value::Atom(s.clone()),
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
            | ExprKind::Await { .. }
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
        assert_eq!(eval_literal(&Literal::Float(3.15)), Value::Float(3.15));
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

    #[test]
    fn test_is_assignment_expr() {
        let assign_expr = ExprKind::Assign {
            target: Box::new(make_literal_expr(Literal::Integer(1, None))),
            value: Box::new(make_literal_expr(Literal::Integer(2, None))),
        };
        assert!(is_assignment_expr(&assign_expr));

        let literal_expr = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!is_assignment_expr(&literal_expr));
    }

    #[test]
    fn test_eval_return_expr_with_value() {
        let value_expr = make_literal_expr(Literal::Integer(42, None));
        let result = eval_return_expr(Some(&value_expr), |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        });

        match result {
            Err(InterpreterError::Return(Value::Integer(42))) => {}
            _ => panic!("Expected Return error with integer 42"),
        }
    }

    #[test]
    fn test_eval_return_expr_without_value() {
        let result = eval_return_expr(None, |_| Ok(Value::Nil));

        match result {
            Err(InterpreterError::Return(Value::Nil)) => {}
            _ => panic!("Expected Return error with Nil"),
        }
    }

    #[test]
    fn test_eval_array_init_expr() {
        let value_expr = make_literal_expr(Literal::Integer(0, None));
        let size_expr = make_literal_expr(Literal::Integer(3, None));

        let result = eval_array_init_expr(&value_expr, &size_expr, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        match result {
            Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], Value::Integer(0));
                assert_eq!(arr[1], Value::Integer(0));
                assert_eq!(arr[2], Value::Integer(0));
            }
            _ => panic!("Expected array value"),
        }
    }

    #[test]
    fn test_eval_array_init_expr_invalid_size() {
        let value_expr = make_literal_expr(Literal::Integer(0, None));
        let size_expr = make_literal_expr(Literal::String("not a number".to_string()));

        let result = eval_array_init_expr(&value_expr, &size_expr, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            ExprKind::Literal(Literal::String(s)) => Ok(Value::from_string(s.clone())),
            _ => Ok(Value::Nil),
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_eval_block_expr() {
        let statements = vec![
            make_literal_expr(Literal::Integer(1, None)),
            make_literal_expr(Literal::Integer(2, None)),
            make_literal_expr(Literal::Integer(3, None)),
        ];

        let result = eval_block_expr(&statements, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        assert_eq!(result, Value::Integer(3)); // Returns last expression
    }

    #[test]
    fn test_eval_block_expr_empty() {
        let statements: Vec<Expr> = vec![];
        let result = eval_block_expr(&statements, |_| Ok(Value::Nil)).unwrap();
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_tuple_expr() {
        let elements = vec![
            make_literal_expr(Literal::Integer(1, None)),
            make_literal_expr(Literal::Bool(true)),
        ];

        let result = eval_tuple_expr(&elements, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            ExprKind::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        match result {
            Value::Tuple(arr) => {
                assert_eq!(arr.len(), 2);
                assert_eq!(arr[0], Value::Integer(1));
                assert_eq!(arr[1], Value::Bool(true));
            }
            _ => panic!("Expected tuple value"),
        }
    }

    #[test]
    fn test_eval_range_expr() {
        let start = make_literal_expr(Literal::Integer(1, None));
        let end = make_literal_expr(Literal::Integer(10, None));

        let result = eval_range_expr(&start, &end, false, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        match result {
            Value::Range {
                start,
                end,
                inclusive,
            } => {
                assert_eq!(*start, Value::Integer(1));
                assert_eq!(*end, Value::Integer(10));
                assert!(!inclusive);
            }
            _ => panic!("Expected range value"),
        }
    }

    #[test]
    fn test_eval_range_expr_inclusive() {
        let start = make_literal_expr(Literal::Integer(0, None));
        let end = make_literal_expr(Literal::Integer(5, None));

        let result = eval_range_expr(&start, &end, true, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        match result {
            Value::Range { inclusive, .. } => {
                assert!(inclusive);
            }
            _ => panic!("Expected range value"),
        }
    }

    #[test]
    fn test_eval_range_expr_invalid_bounds() {
        let start = make_literal_expr(Literal::String("a".to_string()));
        let end = make_literal_expr(Literal::Integer(10, None));

        let result = eval_range_expr(&start, &end, false, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            ExprKind::Literal(Literal::String(s)) => Ok(Value::from_string(s.clone())),
            _ => Ok(Value::Nil),
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_eval_literal_char() {
        assert_eq!(
            eval_literal(&Literal::Char('x')),
            Value::from_string("x".to_string())
        );
    }

    #[test]
    fn test_eval_literal_byte() {
        assert_eq!(eval_literal(&Literal::Byte(255)), Value::Byte(255));
    }

    #[test]
    fn test_eval_literal_atom() {
        let result = eval_literal(&Literal::Atom("ok".to_string()));
        match result {
            Value::Atom(s) => assert_eq!(s, "ok"),
            _ => panic!("Expected atom value"),
        }
    }

    #[test]
    fn test_eval_if_expr_false_no_else() {
        let condition = make_literal_expr(Literal::Bool(false));
        let then_branch = make_literal_expr(Literal::Integer(1, None));

        let result = eval_if_expr(&condition, &then_branch, None, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_if_expr_false_with_else() {
        let condition = make_literal_expr(Literal::Bool(false));
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

        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_is_control_flow_ternary() {
        let ternary_expr = ExprKind::Ternary {
            condition: Box::new(make_literal_expr(Literal::Bool(true))),
            true_expr: Box::new(make_literal_expr(Literal::Integer(1, None))),
            false_expr: Box::new(make_literal_expr(Literal::Integer(2, None))),
        };
        assert!(is_control_flow_expr(&ternary_expr));
    }

    #[test]
    fn test_is_control_flow_match() {
        let match_expr = ExprKind::Match {
            expr: Box::new(make_literal_expr(Literal::Integer(1, None))),
            arms: vec![],
        };
        assert!(is_control_flow_expr(&match_expr));
    }

    #[test]
    fn test_is_data_structure_range() {
        let range_expr = ExprKind::Range {
            start: Box::new(make_literal_expr(Literal::Integer(0, None))),
            end: Box::new(make_literal_expr(Literal::Integer(10, None))),
            inclusive: false,
        };
        assert!(is_data_structure_expr(&range_expr));
    }

    // ============================================================================
    // EXTREME TDD Round 131: Comprehensive eval_expr coverage tests
    // Target: 82.86% → 95%+ coverage
    // ============================================================================

    // --- eval_expr_kind tests ---
    #[test]
    fn test_eval_expr_kind_literal_integer() {
        let lit = Literal::Integer(42, None);
        let result = eval_expr_kind(
            &ExprKind::Literal(lit),
            |_| Ok(Value::Nil),
            |_| Err(InterpreterError::RuntimeError("unused".to_string())),
            |_, _, _| Ok(Value::Nil),
            |_, _| Ok(Value::Nil),
        )
        .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_expr_kind_identifier_found() {
        let result = eval_expr_kind(
            &ExprKind::Identifier("x".to_string()),
            |_| Ok(Value::Nil),
            |name| {
                if name == "x" {
                    Ok(Value::Integer(100))
                } else {
                    Err(InterpreterError::RuntimeError("not found".to_string()))
                }
            },
            |_, _, _| Ok(Value::Nil),
            |_, _| Ok(Value::Nil),
        )
        .unwrap();
        assert_eq!(result, Value::Integer(100));
    }

    #[test]
    fn test_eval_expr_kind_identifier_not_found() {
        let result = eval_expr_kind(
            &ExprKind::Identifier("unknown".to_string()),
            |_| Ok(Value::Nil),
            |_| Err(InterpreterError::RuntimeError("Variable not found".to_string())),
            |_, _, _| Ok(Value::Nil),
            |_, _| Ok(Value::Nil),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_eval_expr_kind_function_named() {
        let body = make_literal_expr(Literal::Integer(1, None));
        let func_expr = ExprKind::Function {
            name: "test".to_string(),
            type_params: vec![],
            params: vec![],
            body: Box::new(body),
            return_type: None,
            is_async: false,
            is_pub: false,
        };
        let result = eval_expr_kind(
            &func_expr,
            |_| Ok(Value::Nil),
            |_| Err(InterpreterError::RuntimeError("unused".to_string())),
            |name, _params, _body| {
                assert_eq!(name.as_ref().map(|s| s.as_str()), Some("test"));
                Ok(Value::from_string("function".to_string()))
            },
            |_, _| Ok(Value::Nil),
        )
        .unwrap();
        assert_eq!(result, Value::from_string("function".to_string()));
    }

    #[test]
    fn test_eval_expr_kind_lambda_empty_params() {
        let body = make_literal_expr(Literal::Integer(1, None));
        let lambda_expr = ExprKind::Lambda {
            params: vec![],
            body: Box::new(body),
        };
        let result = eval_expr_kind(
            &lambda_expr,
            |_| Ok(Value::Nil),
            |_| Err(InterpreterError::RuntimeError("unused".to_string())),
            |_, _, _| Ok(Value::Nil),
            |_params, _body| Ok(Value::from_string("lambda".to_string())),
        )
        .unwrap();
        assert_eq!(result, Value::from_string("lambda".to_string()));
    }

    #[test]
    fn test_eval_expr_kind_binary_returns_error() {
        // Test with an expression that requires interpreter context
        let binary_expr = ExprKind::Binary {
            left: Box::new(make_literal_expr(Literal::Integer(1, None))),
            op: crate::frontend::ast::BinaryOp::Add,
            right: Box::new(make_literal_expr(Literal::Integer(2, None))),
        };
        let result = eval_expr_kind(
            &binary_expr,
            |_| Ok(Value::Nil),
            |_| Err(InterpreterError::RuntimeError("unused".to_string())),
            |_, _, _| Ok(Value::Nil),
            |_, _| Ok(Value::Nil),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("interpreter context"));
    }

    // --- eval_let_expr tests ---
    #[test]
    fn test_eval_let_expr_unit_body_returns_value() {
        let value = make_literal_expr(Literal::Integer(42, None));
        let body = make_literal_expr(Literal::Unit);
        let mut env_var: Option<(String, Value)> = None;

        let result = eval_let_expr(
            "x",
            &value,
            &body,
            |expr| match &expr.kind {
                ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
                ExprKind::Literal(Literal::Unit) => Ok(Value::Nil),
                _ => Ok(Value::Nil),
            },
            |name, val| {
                env_var = Some((name, val));
            },
        )
        .unwrap();

        assert_eq!(result, Value::Integer(42));
        assert_eq!(env_var, Some(("x".to_string(), Value::Integer(42))));
    }

    #[test]
    fn test_eval_let_expr_non_unit_body_evaluates_body() {
        let value = make_literal_expr(Literal::Integer(42, None));
        let body = make_literal_expr(Literal::Integer(100, None));

        let result = eval_let_expr(
            "x",
            &value,
            &body,
            |expr| match &expr.kind {
                ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
                _ => Ok(Value::Nil),
            },
            |_name, _val| {},
        )
        .unwrap();

        assert_eq!(result, Value::Integer(100));
    }

    // --- eval_list_expr tests ---
    #[test]
    fn test_eval_list_expr_empty_array() {
        let result = eval_list_expr(&[], |_| Ok(Value::Nil)).unwrap();
        match result {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected array"),
        }
    }

    #[test]
    fn test_eval_list_expr_error_in_element() {
        let elements = vec![make_literal_expr(Literal::Integer(1, None))];
        let result = eval_list_expr(&elements, |_| {
            Err(InterpreterError::RuntimeError("eval error".to_string()))
        });
        assert!(result.is_err());
    }

    // --- eval_array_init_expr tests ---
    #[test]
    fn test_eval_array_init_expr_zero_size_empty() {
        let value = make_literal_expr(Literal::Integer(42, None));
        let size = make_literal_expr(Literal::Integer(0, None));

        let result = eval_array_init_expr(&value, &size, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        })
        .unwrap();

        match result {
            Value::Array(arr) => assert_eq!(arr.len(), 0),
            _ => panic!("Expected empty array"),
        }
    }

    // --- eval_block_expr tests ---
    #[test]
    fn test_eval_block_expr_single_stmt() {
        let stmts = vec![make_literal_expr(Literal::Integer(42, None))];
        let result = eval_block_expr(&stmts, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        })
        .unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_block_expr_error_stops() {
        let stmts = vec![make_literal_expr(Literal::Integer(1, None))];
        let result = eval_block_expr(&stmts, |_| {
            Err(InterpreterError::RuntimeError("block error".to_string()))
        });
        assert!(result.is_err());
    }

    // --- eval_tuple_expr tests ---
    #[test]
    fn test_eval_tuple_expr_empty_tuple() {
        let result = eval_tuple_expr(&[], |_| Ok(Value::Nil)).unwrap();
        match result {
            Value::Tuple(t) => assert_eq!(t.len(), 0),
            _ => panic!("Expected tuple"),
        }
    }

    #[test]
    fn test_eval_tuple_expr_error_stops() {
        let elements = vec![make_literal_expr(Literal::Integer(1, None))];
        let result = eval_tuple_expr(&elements, |_| {
            Err(InterpreterError::RuntimeError("tuple error".to_string()))
        });
        assert!(result.is_err());
    }

    // --- is_assignment_expr tests ---
    #[test]
    fn test_is_assignment_expr_compound_add() {
        let compound_expr = ExprKind::CompoundAssign {
            target: Box::new(make_literal_expr(Literal::Integer(1, None))),
            op: crate::frontend::ast::BinaryOp::Add,
            value: Box::new(make_literal_expr(Literal::Integer(2, None))),
        };
        assert!(is_assignment_expr(&compound_expr));
    }

    #[test]
    fn test_is_assignment_expr_literal_false() {
        let lit_expr = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!is_assignment_expr(&lit_expr));
    }

    // --- Additional control flow tests ---
    #[test]
    fn test_is_control_flow_let_binding() {
        let let_expr = ExprKind::Let {
            name: "x".to_string(),
            value: Box::new(make_literal_expr(Literal::Integer(1, None))),
            body: Box::new(make_literal_expr(Literal::Unit)),
            type_annotation: None,
            is_mutable: false,
            else_block: None,
        };
        assert!(is_control_flow_expr(&let_expr));
    }

    #[test]
    fn test_is_control_flow_for_loop() {
        let for_expr = ExprKind::For {
            label: None,
            var: "i".to_string(),
            pattern: None,
            iter: Box::new(make_literal_expr(Literal::Integer(1, None))),
            body: Box::new(make_literal_expr(Literal::Unit)),
        };
        assert!(is_control_flow_expr(&for_expr));
    }

    #[test]
    fn test_is_control_flow_while_loop() {
        let while_expr = ExprKind::While {
            label: None,
            condition: Box::new(make_literal_expr(Literal::Bool(true))),
            body: Box::new(make_literal_expr(Literal::Unit)),
        };
        assert!(is_control_flow_expr(&while_expr));
    }

    #[test]
    fn test_is_control_flow_infinite_loop() {
        let loop_expr = ExprKind::Loop {
            label: None,
            body: Box::new(make_literal_expr(Literal::Unit)),
        };
        assert!(is_control_flow_expr(&loop_expr));
    }

    #[test]
    fn test_is_control_flow_break_no_value() {
        let break_expr = ExprKind::Break {
            label: None,
            value: None,
        };
        assert!(is_control_flow_expr(&break_expr));
    }

    #[test]
    fn test_is_control_flow_continue_loop() {
        let continue_expr = ExprKind::Continue { label: None };
        assert!(is_control_flow_expr(&continue_expr));
    }

    #[test]
    fn test_is_control_flow_return_void() {
        let return_expr = ExprKind::Return { value: None };
        assert!(is_control_flow_expr(&return_expr));
    }

    #[test]
    fn test_is_control_flow_try_catch_block() {
        let try_expr = ExprKind::TryCatch {
            try_block: Box::new(make_literal_expr(Literal::Unit)),
            catch_clauses: vec![],
            finally_block: None,
        };
        assert!(is_control_flow_expr(&try_expr));
    }

    #[test]
    fn test_is_control_flow_throw_error() {
        let throw_expr = ExprKind::Throw {
            expr: Box::new(make_literal_expr(Literal::String("error".to_string()))),
        };
        assert!(is_control_flow_expr(&throw_expr));
    }

    // --- Additional data structure tests ---
    #[test]
    fn test_is_data_structure_block_expr() {
        let block_expr = ExprKind::Block(vec![]);
        assert!(is_data_structure_expr(&block_expr));
    }

    #[test]
    fn test_is_data_structure_tuple_expr() {
        let tuple_expr = ExprKind::Tuple(vec![make_literal_expr(Literal::Integer(1, None))]);
        assert!(is_data_structure_expr(&tuple_expr));
    }

    #[test]
    fn test_is_data_structure_array_init_expr() {
        let array_init = ExprKind::ArrayInit {
            value: Box::new(make_literal_expr(Literal::Integer(0, None))),
            size: Box::new(make_literal_expr(Literal::Integer(10, None))),
        };
        assert!(is_data_structure_expr(&array_init));
    }

    #[test]
    fn test_is_data_structure_dataframe_expr() {
        let df_expr = ExprKind::DataFrame { columns: vec![] };
        assert!(is_data_structure_expr(&df_expr));
    }

    #[test]
    fn test_is_data_structure_list_expr() {
        let list_expr = ExprKind::List(vec![
            make_literal_expr(Literal::Integer(1, None)),
            make_literal_expr(Literal::Integer(2, None)),
        ]);
        assert!(is_data_structure_expr(&list_expr));
    }

    #[test]
    fn test_is_data_structure_call_is_false() {
        let call_expr = ExprKind::Call {
            func: Box::new(make_literal_expr(Literal::Integer(1, None))),
            args: vec![],
        };
        assert!(!is_data_structure_expr(&call_expr));
    }

    // --- eval_if_expr additional tests ---
    #[test]
    fn test_eval_if_expr_truthy_nonzero() {
        let condition = make_literal_expr(Literal::Integer(1, None));
        let then_branch = make_literal_expr(Literal::String("yes".to_string()));
        let else_branch = make_literal_expr(Literal::String("no".to_string()));

        let result = eval_if_expr(
            &condition,
            &then_branch,
            Some(&else_branch),
            |expr| match &expr.kind {
                ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
                ExprKind::Literal(Literal::String(s)) => Ok(Value::from_string(s.clone())),
                _ => Ok(Value::Nil),
            },
        )
        .unwrap();

        assert_eq!(result, Value::from_string("yes".to_string()));
    }

    #[test]
    fn test_eval_if_expr_falsy_bool_takes_else() {
        let condition = make_literal_expr(Literal::Bool(false));
        let then_branch = make_literal_expr(Literal::String("yes".to_string()));
        let else_branch = make_literal_expr(Literal::String("no".to_string()));

        let result = eval_if_expr(
            &condition,
            &then_branch,
            Some(&else_branch),
            |expr| match &expr.kind {
                ExprKind::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
                ExprKind::Literal(Literal::String(s)) => Ok(Value::from_string(s.clone())),
                _ => Ok(Value::Nil),
            },
        )
        .unwrap();

        assert_eq!(result, Value::from_string("no".to_string()));
    }

    #[test]
    fn test_eval_if_expr_condition_error_propagates() {
        let condition = make_literal_expr(Literal::Integer(1, None));
        let then_branch = make_literal_expr(Literal::Integer(1, None));

        let result = eval_if_expr(&condition, &then_branch, None, |_| {
            Err(InterpreterError::RuntimeError("condition error".to_string()))
        });

        assert!(result.is_err());
    }

    // === EXTREME TDD Round 136 - Push to 70+ Tests ===

    #[test]
    fn test_eval_literal_float_zero() {
        let lit = Literal::Float(0.0);
        let result = eval_literal(&lit);
        assert_eq!(result, Value::Float(0.0));
    }

    #[test]
    fn test_eval_literal_float_negative() {
        let lit = Literal::Float(-3.14);
        let result = eval_literal(&lit);
        assert_eq!(result, Value::Float(-3.14));
    }

    #[test]
    fn test_eval_literal_string_empty() {
        let lit = Literal::String(String::new());
        let result = eval_literal(&lit);
        if let Value::String(s) = result {
            assert!(s.is_empty());
        } else {
            panic!("Expected String");
        }
    }

    #[test]
    fn test_eval_literal_bool_true() {
        let lit = Literal::Bool(true);
        let result = eval_literal(&lit);
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_literal_byte_zero() {
        let lit = Literal::Byte(0);
        let result = eval_literal(&lit);
        assert_eq!(result, Value::Byte(0));
    }

    #[test]
    fn test_eval_literal_byte_max() {
        let lit = Literal::Byte(255);
        let result = eval_literal(&lit);
        assert_eq!(result, Value::Byte(255));
    }

    #[test]
    fn test_is_control_flow_for() {
        let for_expr = ExprKind::For {
            label: None,
            var: "i".to_string(),
            pattern: None,
            iter: Box::new(make_literal_expr(Literal::Integer(1, None))),
            body: Box::new(make_literal_expr(Literal::Integer(1, None))),
        };
        assert!(is_control_flow_expr(&for_expr));
    }

    #[test]
    fn test_is_control_flow_while() {
        let while_expr = ExprKind::While {
            label: None,
            condition: Box::new(make_literal_expr(Literal::Bool(true))),
            body: Box::new(make_literal_expr(Literal::Integer(1, None))),
        };
        assert!(is_control_flow_expr(&while_expr));
    }

    #[test]
    fn test_is_control_flow_loop() {
        let loop_expr = ExprKind::Loop {
            label: None,
            body: Box::new(make_literal_expr(Literal::Integer(1, None))),
        };
        assert!(is_control_flow_expr(&loop_expr));
    }

    #[test]
    fn test_is_control_flow_break() {
        let break_expr = ExprKind::Break { label: None, value: None };
        assert!(is_control_flow_expr(&break_expr));
    }

    #[test]
    fn test_is_control_flow_continue() {
        let continue_expr = ExprKind::Continue { label: None };
        assert!(is_control_flow_expr(&continue_expr));
    }

    #[test]
    fn test_is_control_flow_return() {
        let return_expr = ExprKind::Return { value: None };
        assert!(is_control_flow_expr(&return_expr));
    }

    #[test]
    fn test_is_control_flow_identifier_is_false() {
        let ident_expr = ExprKind::Identifier("x".to_string());
        assert!(!is_control_flow_expr(&ident_expr));
    }

    #[test]
    fn test_is_assignment_compound_assign() {
        let compound = ExprKind::CompoundAssign {
            target: Box::new(make_literal_expr(Literal::Integer(1, None))),
            op: crate::frontend::ast::BinaryOp::Add,
            value: Box::new(make_literal_expr(Literal::Integer(1, None))),
        };
        assert!(is_assignment_expr(&compound));
    }

    #[test]
    fn test_is_assignment_literal_is_false() {
        let lit = ExprKind::Literal(Literal::Integer(42, None));
        assert!(!is_assignment_expr(&lit));
    }

    #[test]
    fn test_eval_list_expr_single() {
        let elements = vec![make_literal_expr(Literal::Integer(42, None))];
        let result = eval_list_expr(&elements, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        })
        .unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 1);
            assert_eq!(arr[0], Value::Integer(42));
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_return_expr_with_int_value() {
        let value = make_literal_expr(Literal::Integer(42, None));
        let result = eval_return_expr(Some(&value), |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        });
        assert!(result.is_err());
        if let Err(InterpreterError::Return(val)) = result {
            assert_eq!(val, Value::Integer(42));
        }
    }

    #[test]
    fn test_eval_if_expr_integer_is_truthy() {
        // Integer values are truthy in value_utils (falls into _ => true)
        let condition = make_literal_expr(Literal::Integer(0, None));
        let then_branch = make_literal_expr(Literal::String("yes".to_string()));
        let else_branch = make_literal_expr(Literal::String("no".to_string()));

        let result = eval_if_expr(
            &condition,
            &then_branch,
            Some(&else_branch),
            |expr| match &expr.kind {
                ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
                ExprKind::Literal(Literal::String(s)) => Ok(Value::from_string(s.clone())),
                _ => Ok(Value::Nil),
            },
        )
        .unwrap();

        // Integer(0) is truthy in value_utils.rs (falls into _ => true)
        assert_eq!(result, Value::from_string("yes".to_string()));
    }

    // === EXTREME TDD Round 159 - Coverage Push Tests ===

    #[test]
    fn test_eval_literal_integer_r159() {
        let lit = Literal::Integer(42, None);
        assert_eq!(eval_literal(&lit), Value::Integer(42));
    }

    #[test]
    fn test_eval_literal_float_r159() {
        let lit = Literal::Float(3.14);
        assert_eq!(eval_literal(&lit), Value::Float(3.14));
    }

    #[test]
    fn test_eval_literal_string_r159() {
        let lit = Literal::String("hello".to_string());
        let result = eval_literal(&lit);
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_eval_literal_bool_r159() {
        assert_eq!(eval_literal(&Literal::Bool(true)), Value::Bool(true));
        assert_eq!(eval_literal(&Literal::Bool(false)), Value::Bool(false));
    }

    #[test]
    fn test_eval_literal_char_r159() {
        let lit = Literal::Char('x');
        let result = eval_literal(&lit);
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_eval_literal_byte_r159() {
        let lit = Literal::Byte(255);
        assert_eq!(eval_literal(&lit), Value::Byte(255));
    }

    #[test]
    fn test_eval_literal_unit_r159() {
        assert_eq!(eval_literal(&Literal::Unit), Value::Nil);
    }

    #[test]
    fn test_eval_literal_null_r159() {
        assert_eq!(eval_literal(&Literal::Null), Value::Nil);
    }

    #[test]
    fn test_eval_literal_atom_r159() {
        let lit = Literal::Atom("ok".to_string());
        let result = eval_literal(&lit);
        assert!(matches!(result, Value::Atom(_)));
    }

    #[test]
    fn test_is_control_flow_if_r159() {
        let expr_kind = ExprKind::If {
            condition: Box::new(make_literal_expr(Literal::Bool(true))),
            then_branch: Box::new(make_literal_expr(Literal::Integer(1, None))),
            else_branch: None,
        };
        assert!(is_control_flow_expr(&expr_kind));
    }

    #[test]
    fn test_is_control_flow_while_r159() {
        let expr_kind = ExprKind::While {
            condition: Box::new(make_literal_expr(Literal::Bool(true))),
            body: Box::new(make_literal_expr(Literal::Unit)),
            label: None,
        };
        assert!(is_control_flow_expr(&expr_kind));
    }

    #[test]
    fn test_is_control_flow_loop_r159() {
        let expr_kind = ExprKind::Loop {
            body: Box::new(make_literal_expr(Literal::Unit)),
            label: None,
        };
        assert!(is_control_flow_expr(&expr_kind));
    }

    #[test]
    fn test_is_control_flow_break_r159() {
        let expr_kind = ExprKind::Break { label: None, value: None };
        assert!(is_control_flow_expr(&expr_kind));
    }

    #[test]
    fn test_is_control_flow_continue_r159() {
        let expr_kind = ExprKind::Continue { label: None };
        assert!(is_control_flow_expr(&expr_kind));
    }

    #[test]
    fn test_is_control_flow_return_r159() {
        let expr_kind = ExprKind::Return { value: None };
        assert!(is_control_flow_expr(&expr_kind));
    }

    #[test]
    fn test_is_data_structure_list_r159() {
        let expr_kind = ExprKind::List(vec![]);
        assert!(is_data_structure_expr(&expr_kind));
    }

    #[test]
    fn test_is_data_structure_block_r159() {
        let expr_kind = ExprKind::Block(vec![]);
        assert!(is_data_structure_expr(&expr_kind));
    }

    #[test]
    fn test_is_data_structure_tuple_r159() {
        let expr_kind = ExprKind::Tuple(vec![]);
        assert!(is_data_structure_expr(&expr_kind));
    }

    #[test]
    fn test_is_data_structure_range_r159() {
        let expr_kind = ExprKind::Range {
            start: Box::new(make_literal_expr(Literal::Integer(0, None))),
            end: Box::new(make_literal_expr(Literal::Integer(10, None))),
            inclusive: false,
        };
        assert!(is_data_structure_expr(&expr_kind));
    }

    #[test]
    fn test_is_assignment_assign_r159() {
        let expr_kind = ExprKind::Assign {
            target: Box::new(make_literal_expr(Literal::Unit)),
            value: Box::new(make_literal_expr(Literal::Integer(1, None))),
        };
        assert!(is_assignment_expr(&expr_kind));
    }

    #[test]
    fn test_not_control_flow_r159() {
        let expr_kind = ExprKind::Literal(Literal::Integer(1, None));
        assert!(!is_control_flow_expr(&expr_kind));
    }

    #[test]
    fn test_not_data_structure_r159() {
        let expr_kind = ExprKind::Identifier("x".to_string());
        assert!(!is_data_structure_expr(&expr_kind));
    }

    #[test]
    fn test_not_assignment_r159() {
        let expr_kind = ExprKind::Literal(Literal::Bool(true));
        assert!(!is_assignment_expr(&expr_kind));
    }

    #[test]
    fn test_eval_if_expr_no_else_r159() {
        let condition = make_literal_expr(Literal::Bool(false));
        let then_branch = make_literal_expr(Literal::Integer(1, None));

        let result = eval_if_expr(
            &condition,
            &then_branch,
            None,
            |expr| match &expr.kind {
                ExprKind::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
                ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
                _ => Ok(Value::Nil),
            },
        ).unwrap();

        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_eval_return_expr_none_r159() {
        let result = eval_return_expr(None, |_| Ok(Value::Nil));
        assert!(result.is_err());
        if let Err(InterpreterError::Return(val)) = result {
            assert_eq!(val, Value::Nil);
        }
    }

    #[test]
    fn test_eval_list_expr_empty_r159() {
        let elements: Vec<Expr> = vec![];
        let result = eval_list_expr(&elements, |_| Ok(Value::Nil)).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 0);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_list_expr_multiple_r159() {
        let elements = vec![
            make_literal_expr(Literal::Integer(1, None)),
            make_literal_expr(Literal::Integer(2, None)),
            make_literal_expr(Literal::Integer(3, None)),
        ];
        let result = eval_list_expr(&elements, |expr| match &expr.kind {
            ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
            _ => Ok(Value::Nil),
        }).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected Array");
        }
    }

    #[test]
    fn test_eval_let_expr_unit_body_r159() {
        let value_expr = make_literal_expr(Literal::Integer(42, None));
        let body = make_literal_expr(Literal::Unit);
        let mut bound_var: Option<(String, Value)> = None;

        let result = eval_let_expr(
            "x",
            &value_expr,
            &body,
            |expr| match &expr.kind {
                ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
                ExprKind::Literal(Literal::Unit) => Ok(Value::Nil),
                _ => Ok(Value::Nil),
            },
            |name, val| bound_var = Some((name, val)),
        ).unwrap();

        // With Unit body, let returns the value
        assert_eq!(result, Value::Integer(42));
        assert_eq!(bound_var, Some(("x".to_string(), Value::Integer(42))));
    }

    #[test]
    fn test_eval_let_expr_with_body_r159() {
        let value_expr = make_literal_expr(Literal::Integer(10, None));
        let body = make_literal_expr(Literal::Integer(20, None));

        let result = eval_let_expr(
            "y",
            &value_expr,
            &body,
            |expr| match &expr.kind {
                ExprKind::Literal(Literal::Integer(i, None)) => Ok(Value::Integer(*i)),
                _ => Ok(Value::Nil),
            },
            |_, _| {},
        ).unwrap();

        // With non-Unit body, let returns the body result
        assert_eq!(result, Value::Integer(20));
    }
}
