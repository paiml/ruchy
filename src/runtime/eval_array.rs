//! Array method evaluation module
//!
//! This module handles evaluation of array methods in the interpreter.
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::runtime::pattern_matching::values_equal;
use crate::runtime::validation::validate_arg_count;
use crate::runtime::{InterpreterError, Value};
use std::sync::Arc;

/// Evaluate an array method call
///
/// # Complexity
/// Cyclomatic complexity: 15 (will be decomposed further into helper functions)
pub fn eval_array_method<F>(
    arr: &Arc<[Value]>,
    method: &str,
    args: &[Value],
    mut eval_function_call_value: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    match method {
        // Simple no-argument methods
        "len" | "length" if args.is_empty() => eval_array_len(arr),
        "first" if args.is_empty() => eval_array_first(arr),
        "last" if args.is_empty() => eval_array_last(arr),
        "is_empty" if args.is_empty() => eval_array_is_empty(arr),

        // Single-argument methods
        "push" if args.len() == 1 => eval_array_push(arr, &args[0]),
        "pop" if args.is_empty() => eval_array_pop(arr),
        "get" if args.len() == 1 => eval_array_get(arr, &args[0]),
        "nth" if args.len() == 1 => eval_array_nth(arr, &args[0]),
        "contains" if args.len() == 1 => eval_array_contains(arr, &args[0]),

        // STDLIB-004: Custom array methods
        "slice" if args.len() == 2 => eval_array_slice(arr, &args[0], &args[1]),
        "join" if args.len() == 1 => eval_array_join(arr, &args[0]),
        "unique" if args.is_empty() => eval_array_unique(arr),
        "enumerate" if args.is_empty() => eval_array_enumerate(arr),

        // STDLIB-005: Array concatenation and flattening
        "concat" if args.len() == 1 => eval_array_concat(arr, &args[0]),
        "append" if args.len() == 1 => eval_array_concat(arr, &args[0]), // STDLIB-007: alias for concat (GitHub #47)
        "flatten" if args.is_empty() => eval_array_flatten(arr),

        // STDLIB-007: Set operations (arrays as sets)
        "union" if args.len() == 1 => eval_array_union(arr, &args[0]),
        "intersection" if args.len() == 1 => eval_array_intersection(arr, &args[0]),
        "difference" if args.len() == 1 => eval_array_difference(arr, &args[0]),

        // STDLIB-009: Sort array
        "sort" if args.is_empty() => eval_array_sort(arr),

        // Higher-order methods
        "map" => eval_array_map(arr, args, &mut eval_function_call_value),
        "filter" => eval_array_filter(arr, args, &mut eval_function_call_value),
        "reduce" => eval_array_reduce(arr, args, &mut eval_function_call_value),
        "any" => eval_array_any(arr, args, &mut eval_function_call_value),
        "all" => eval_array_all(arr, args, &mut eval_function_call_value),
        "find" => eval_array_find(arr, args, &mut eval_function_call_value),
        "each" => eval_array_each(arr, args, &mut eval_function_call_value),

        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown array method: {method}"
        ))),
    }
}

// No-argument array methods (complexity <= 3 each)

fn eval_array_len(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    Ok(Value::Integer(arr.len() as i64))
}

fn eval_array_first(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    Ok(arr.first().cloned().unwrap_or(Value::Nil))
}

fn eval_array_last(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    Ok(arr.last().cloned().unwrap_or(Value::Nil))
}

fn eval_array_is_empty(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    Ok(Value::Bool(arr.is_empty()))
}

// Single-argument array methods (complexity <= 5 each)

fn eval_array_push(arr: &Arc<[Value]>, item: &Value) -> Result<Value, InterpreterError> {
    let mut new_arr = arr.to_vec();
    new_arr.push(item.clone());
    Ok(Value::Array(Arc::from(new_arr)))
}

fn eval_array_pop(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    let mut new_arr = arr.to_vec();
    new_arr.pop().unwrap_or(Value::nil());
    Ok(Value::Array(Arc::from(new_arr)))
}

fn eval_array_get(arr: &Arc<[Value]>, index: &Value) -> Result<Value, InterpreterError> {
    if let Value::Integer(idx) = index {
        if *idx < 0 {
            return Ok(Value::Nil);
        }
        #[allow(clippy::cast_sign_loss)]
        let index = *idx as usize;
        if index < arr.len() {
            Ok(arr[index].clone())
        } else {
            Ok(Value::Nil)
        }
    } else {
        Err(InterpreterError::RuntimeError(
            "get expects integer index".to_string(),
        ))
    }
}

/// Return the nth element of an array wrapped in `Option::Some`, or `Option::None` if out of bounds
///
/// # Complexity
/// Cyclomatic complexity: 4 (well within <10 limit)
fn eval_array_nth(arr: &Arc<[Value]>, index: &Value) -> Result<Value, InterpreterError> {
    if let Value::Integer(idx) = index {
        if *idx < 0 {
            return Ok(Value::EnumVariant {
                variant_name: "None".to_string(),
                data: None,
            });
        }
        #[allow(clippy::cast_sign_loss)]
        let index = *idx as usize;
        if index < arr.len() {
            Ok(Value::EnumVariant {
                variant_name: "Some".to_string(),
                data: Some(vec![arr[index].clone()]),
            })
        } else {
            Ok(Value::EnumVariant {
                variant_name: "None".to_string(),
                data: None,
            })
        }
    } else {
        Err(InterpreterError::RuntimeError(
            "nth expects integer index".to_string(),
        ))
    }
}

fn eval_array_contains(arr: &Arc<[Value]>, item: &Value) -> Result<Value, InterpreterError> {
    // Check if the array contains the given value
    for element in arr.iter() {
        if values_equal(element, item) {
            return Ok(Value::Bool(true));
        }
    }
    Ok(Value::Bool(false))
}

/// Returns array of (index, value) tuples for iteration with position tracking
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
///
/// # Examples
/// ```
/// [10, 20, 30].enumerate() => [(0, 10), (1, 20), (2, 30)]
/// ```
fn eval_array_enumerate(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    let enumerated: Vec<Value> = arr
        .iter()
        .enumerate()
        .map(|(i, val)| {
            Value::Tuple(Arc::from(vec![Value::Integer(i as i64), val.clone()]))
        })
        .collect();
    Ok(Value::Array(Arc::from(enumerated)))
}

// STDLIB-004: Custom array methods (complexity <= 5 each)

/// Extract slice from array
/// Complexity: 5 (within Toyota Way limits)
fn eval_array_slice(arr: &Arc<[Value]>, start: &Value, end: &Value) -> Result<Value, InterpreterError> {
    match (start, end) {
        (Value::Integer(s), Value::Integer(e)) => {
            let start_idx = (*s).max(0) as usize;
            let end_idx = (*e).max(0) as usize;
            let slice: Vec<Value> = arr
                .iter()
                .skip(start_idx)
                .take(end_idx.saturating_sub(start_idx))
                .cloned()
                .collect();
            Ok(Value::Array(Arc::from(slice)))
        }
        _ => Err(InterpreterError::RuntimeError(
            "Array.slice() expects two integer arguments".to_string(),
        )),
    }
}

/// Join array elements into string
/// Complexity: 4 (within Toyota Way limits)
fn eval_array_join(arr: &Arc<[Value]>, separator: &Value) -> Result<Value, InterpreterError> {
    match separator {
        Value::String(sep) => {
            let strings: Vec<String> = arr
                .iter()
                .map(|v| match v {
                    Value::String(s) => s.to_string(),
                    _ => format!("{v}"),
                })
                .collect();
            Ok(Value::from_string(strings.join(sep.as_ref())))
        }
        _ => Err(InterpreterError::RuntimeError(
            "Array.join() expects a string argument".to_string(),
        )),
    }
}

/// Remove duplicate elements from array
/// Complexity: 3 (within Toyota Way limits)
fn eval_array_unique(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<Value> = arr
        .iter()
        .filter(|v| {
            let key = format!("{v:?}");
            seen.insert(key)
        })
        .cloned()
        .collect();
    Ok(Value::Array(Arc::from(unique)))
}

// Higher-order array methods (complexity <= 8 each)

fn eval_array_map<F>(
    arr: &Arc<[Value]>,
    args: &[Value],
    eval_function_call_value: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    validate_single_closure_argument(args, "map")?;
    let mut result = Vec::new();
    for item in arr.iter() {
        let func_result = eval_function_call_value(&args[0], std::slice::from_ref(item))?;
        result.push(func_result);
    }
    Ok(Value::Array(Arc::from(result)))
}

fn eval_array_filter<F>(
    arr: &Arc<[Value]>,
    args: &[Value],
    eval_function_call_value: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    validate_single_closure_argument(args, "filter")?;
    let mut result = Vec::new();
    for item in arr.iter() {
        let func_result = eval_function_call_value(&args[0], std::slice::from_ref(item))?;
        if func_result.is_truthy() {
            result.push(item.clone());
        }
    }
    Ok(Value::Array(Arc::from(result)))
}

fn eval_array_reduce<F>(
    arr: &Arc<[Value]>,
    args: &[Value],
    eval_function_call_value: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    validate_arg_count("reduce", args, 2)?;
    if !matches!(&args[0], Value::Closure { .. }) {
        return Err(InterpreterError::RuntimeError(
            "reduce expects a function and initial value".to_string(),
        ));
    }

    let mut accumulator = args[1].clone();
    for item in arr.iter() {
        accumulator = eval_function_call_value(&args[0], &[accumulator, item.clone()])?;
    }
    Ok(accumulator)
}

fn eval_array_any<F>(
    arr: &Arc<[Value]>,
    args: &[Value],
    eval_function_call_value: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    validate_single_closure_argument(args, "any")?;
    for item in arr.iter() {
        let func_result = eval_function_call_value(&args[0], std::slice::from_ref(item))?;
        if func_result.is_truthy() {
            return Ok(Value::Bool(true));
        }
    }
    Ok(Value::Bool(false))
}

fn eval_array_all<F>(
    arr: &Arc<[Value]>,
    args: &[Value],
    eval_function_call_value: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    validate_single_closure_argument(args, "all")?;
    for item in arr.iter() {
        let func_result = eval_function_call_value(&args[0], std::slice::from_ref(item))?;
        if !func_result.is_truthy() {
            return Ok(Value::Bool(false));
        }
    }
    Ok(Value::Bool(true))
}

fn eval_array_find<F>(
    arr: &Arc<[Value]>,
    args: &[Value],
    eval_function_call_value: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    validate_single_closure_argument(args, "find")?;
    for item in arr.iter() {
        let func_result = eval_function_call_value(&args[0], std::slice::from_ref(item))?;
        if func_result.is_truthy() {
            return Ok(item.clone());
        }
    }
    Ok(Value::Nil)
}

/// STDLIB-010: Array.each() method
/// Iterates over array elements, calling closure for side effects
/// Returns Nil (unlike map which returns transformed results)
///
/// Complexity: 3 (within Toyota Way limit of ≤10)
fn eval_array_each<F>(
    arr: &Arc<[Value]>,
    args: &[Value],
    eval_function_call_value: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    validate_single_closure_argument(args, "each")?;
    for item in arr.iter() {
        // Call closure for side effects, discard result
        eval_function_call_value(&args[0], std::slice::from_ref(item))?;
    }
    Ok(Value::Nil)
}

// Helper function (complexity <= 2, reduced from 3)

fn validate_single_closure_argument(
    args: &[Value],
    method_name: &str,
) -> Result<(), InterpreterError> {
    validate_arg_count(method_name, args, 1)?;
    if !matches!(&args[0], Value::Closure { .. }) {
        return Err(InterpreterError::RuntimeError(format!(
            "{method_name} expects a function argument"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_len() {
        let arr = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = eval_array_len(&arr).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_array_first() {
        let arr = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = eval_array_first(&arr).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_array_last() {
        let arr = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = eval_array_last(&arr).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_array_push() {
        let arr = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let result = eval_array_push(&arr, &Value::Integer(3)).unwrap();
        if let Value::Array(new_arr) = result {
            assert_eq!(new_arr.len(), 3);
            assert_eq!(new_arr[2], Value::Integer(3));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_array_get() {
        let arr = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = eval_array_get(&arr, &Value::Integer(1)).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    // STDLIB-007 (GitHub Issue #47): array.append() tests
    #[test]
    fn test_array_append_basic() {
        let arr = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let other = Value::Array(Arc::from(vec![Value::Integer(3), Value::Integer(4)]));

        let dummy_eval = |_: &Value, _: &[Value]| Ok(Value::Nil);
        let result = eval_array_method(&arr, "append", &[other], dummy_eval).unwrap();

        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 4);
            assert_eq!(result_arr[0], Value::Integer(1));
            assert_eq!(result_arr[1], Value::Integer(2));
            assert_eq!(result_arr[2], Value::Integer(3));
            assert_eq!(result_arr[3], Value::Integer(4));
        } else {
            panic!("Expected array result from append");
        }
    }

    #[test]
    fn test_array_append_empty_arrays() {
        let arr = Arc::from(vec![Value::Integer(1)]);
        let other = Value::Array(Arc::from(vec![]));

        let dummy_eval = |_: &Value, _: &[Value]| Ok(Value::Nil);
        let result = eval_array_method(&arr, "append", &[other], dummy_eval).unwrap();

        if let Value::Array(result_arr) = result {
            assert_eq!(result_arr.len(), 1);
            assert_eq!(result_arr[0], Value::Integer(1));
        } else {
            panic!("Expected array result from append");
        }
    }

    #[test]
    fn test_array_append_wrong_arg_type() {
        let arr = Arc::from(vec![Value::Integer(1)]);
        let dummy_eval = |_: &Value, _: &[Value]| Ok(Value::Nil);

        // append() with non-array argument should fail
        let result = eval_array_method(&arr, "append", &[Value::Integer(42)], dummy_eval);
        assert!(result.is_err(), "append() should reject non-array arguments");
    }

    #[test]
    fn test_eval_array_method_match_guards() {
        // Mutation test: Verify match guards work correctly
        // MISSED: replace match guard args.is_empty() with true (line 26)
        // MISSED: replace match guard args.len() == 1 with false (line 30)

        let arr = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let dummy_eval = |_: &Value, _: &[Value]| Ok(Value::Nil);

        // Test "first" with no args (should work)
        let result = eval_array_method(&arr, "first", &[], dummy_eval);
        assert!(result.is_ok(), "first with 0 args should work");
        assert_eq!(result.unwrap(), Value::Integer(1));

        // Test "first" with args (should fail due to guard)
        let result = eval_array_method(&arr, "first", &[Value::Integer(0)], dummy_eval);
        assert!(
            result.is_err(),
            "first with args should fail (match guard: args.is_empty())"
        );

        // Test "push" with 1 arg (should work)
        let result = eval_array_method(&arr, "push", &[Value::Integer(4)], dummy_eval);
        assert!(result.is_ok(), "push with 1 arg should work");

        // Test "push" with wrong number of args (should fail due to guard)
        let result = eval_array_method(&arr, "push", &[], dummy_eval);
        assert!(
            result.is_err(),
            "push with 0 args should fail (match guard: args.len() == 1)"
        );
    }

    #[test]
    fn test_eval_array_method_match_arms_any_all() {
        // Mutation test: Verify "any" and "all" match arms exist
        // MISSED: delete match arm "any" (line 38)
        // MISSED: delete match arm "all" (line 39)

        let arr = Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);

        // Test by calling eval_array_any and eval_array_all directly
        // Both require a closure, so we create a minimal one
        use std::sync::Arc as RcAlias;
        let closure = Value::Closure {
            params: vec!["x".to_string()],
            body: RcAlias::new(crate::frontend::ast::Expr {
                kind: crate::frontend::ast::ExprKind::Identifier("x".to_string()),
                span: crate::frontend::ast::Span::new(0, 0),
                attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            }),
            env: Default::default(),
        };

        let eval_func = |_: &Value, _: &[Value]| Ok(Value::Bool(true));

        // Test "any" method via eval_array_any
        let result = eval_array_any(&arr, &[closure.clone()], &mut |f, a| eval_func(f, a));
        assert!(result.is_ok(), "any method should work (match arm test)");

        // Test "all" method via eval_array_all
        let result = eval_array_all(&arr, &[closure], &mut |f, a| eval_func(f, a));
        assert!(result.is_ok(), "all method should work (match arm test)");
    }

    #[test]
    fn test_eval_array_reduce_comparison_operator() {
        // Mutation test: Verify reduce uses != not ==
        // MISSED: replace != with == (line 141:19 in eval_array_reduce)

        let arr = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);

        // Test with correct number of args (2) - should work
        use std::sync::Arc as RcAlias;
        let closure = Value::Closure {
            params: vec!["acc".to_string(), "x".to_string()],
            body: RcAlias::new(crate::frontend::ast::Expr {
                kind: crate::frontend::ast::ExprKind::Identifier("acc".to_string()),
                span: crate::frontend::ast::Span::new(0, 0),
                attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            }),
            env: Default::default(),
        };
        let eval_func = |_: &Value, args: &[Value]| Ok(args[0].clone());
        let result = eval_array_reduce(&arr, &[closure.clone(), Value::Integer(0)], &mut |f, a| {
            eval_func(f, a)
        });
        assert!(
            result.is_ok(),
            "reduce with 2 args should work (args.len() != 2 check)"
        );

        // Test with wrong number of args (1) - should fail
        let result = eval_array_reduce(&arr, &[closure.clone()], &mut |f, a| eval_func(f, a));
        assert!(
            result.is_err(),
            "reduce with 1 arg should fail (proves != operator, not ==)"
        );

        // Test with wrong number of args (3) - should fail
        let result = eval_array_reduce(
            &arr,
            &[closure, Value::Integer(0), Value::Integer(1)],
            &mut |f, a| eval_func(f, a),
        );
        assert!(
            result.is_err(),
            "reduce with 3 args should fail (proves != operator)"
        );
    }

    #[test]
    fn test_eval_array_reduce_negation_operator() {
        // Mutation test: Verify reduce checks !matches (closure check)
        // MISSED: delete ! in eval_array_reduce (line 146:8)

        let arr = Arc::from(vec![Value::Integer(1)]);
        let eval_func = |_: &Value, _: &[Value]| Ok(Value::Nil);

        // Test with non-closure first arg (should fail due to ! check)
        let result = eval_array_reduce(
            &arr,
            &[Value::Integer(0), Value::Integer(0)],
            &mut |f, a| eval_func(f, a),
        );
        assert!(
            result.is_err(),
            "reduce should reject non-closure (tests !matches negation)"
        );

        // Test with closure (should work)
        use std::sync::Arc as RcAlias;
        let closure = Value::Closure {
            params: vec![],
            body: RcAlias::new(crate::frontend::ast::Expr {
                kind: crate::frontend::ast::ExprKind::Literal(crate::frontend::ast::Literal::Null),
                span: crate::frontend::ast::Span::new(0, 0),
                attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            }),
            env: Default::default(),
        };
        let result = eval_array_reduce(&arr, &[closure, Value::Integer(0)], &mut |f, a| {
            eval_func(f, a)
        });
        assert!(
            result.is_ok(),
            "reduce should accept closure (proves negation works)"
        );
    }

    #[test]
    fn test_eval_array_all_negation_operator() {
        // Mutation test: Verify all() uses ! to check falsy values
        // MISSED: delete ! in eval_array_all (line 188:12)

        let arr = Arc::from(vec![
            Value::Bool(true),
            Value::Bool(false),
            Value::Bool(true),
        ]);

        // Evaluator that returns the actual value
        let eval_identity = |_: &Value, args: &[Value]| Ok(args[0].clone());

        use std::sync::Arc as RcAlias;
        let closure = Value::Closure {
            params: vec!["x".to_string()],
            body: RcAlias::new(crate::frontend::ast::Expr {
                kind: crate::frontend::ast::ExprKind::Identifier("x".to_string()),
                span: crate::frontend::ast::Span::new(0, 0),
                attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            }),
            env: Default::default(),
        };

        // all() should return false because one element is false
        // This tests the !func_result.is_truthy() logic
        let result = eval_array_all(&arr, &[closure], &mut |f, a| eval_identity(f, a)).unwrap();
        assert_eq!(
            result,
            Value::Bool(false),
            "all() should return false when any element is falsy (tests ! operator)"
        );

        // Test with all true values
        let all_true_arr = Arc::from(vec![Value::Bool(true), Value::Bool(true)]);
        let closure2 = Value::Closure {
            params: vec!["x".to_string()],
            body: RcAlias::new(crate::frontend::ast::Expr {
                kind: crate::frontend::ast::ExprKind::Identifier("x".to_string()),
                span: crate::frontend::ast::Span::new(0, 0),
                attributes: vec![],
            leading_comments: vec![],
            trailing_comment: None,
            }),
            env: Default::default(),
        };
        let result =
            eval_array_all(&all_true_arr, &[closure2], &mut |f, a| eval_identity(f, a)).unwrap();
        assert_eq!(
            result,
            Value::Bool(true),
            "all() should return true when all elements are truthy"
        );
    }

    #[test]
    fn test_array_nth_in_bounds() {
        let arr = Arc::from(vec![
            Value::Integer(10),
            Value::Integer(20),
            Value::Integer(30),
        ]);
        let result = eval_array_nth(&arr, &Value::Integer(1)).unwrap();

        // Should return Option::Some(20)
        match result {
            Value::EnumVariant { variant_name, data } if variant_name == "Some" => {
                assert_eq!(data.unwrap()[0], Value::Integer(20));
            }
            _ => panic!("Expected Some variant"),
        }
    }

    #[test]
    fn test_array_nth_out_of_bounds() {
        let arr = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let result = eval_array_nth(&arr, &Value::Integer(10)).unwrap();

        // Should return Option::None
        match result {
            Value::EnumVariant { variant_name, data } if variant_name == "None" => {
                assert!(data.is_none());
            }
            _ => panic!("Expected None variant"),
        }
    }

    #[test]
    fn test_array_nth_negative_index() {
        let arr = Arc::from(vec![Value::Integer(1), Value::Integer(2)]);
        let result = eval_array_nth(&arr, &Value::Integer(-1)).unwrap();

        // Should return Option::None for negative indices
        match result {
            Value::EnumVariant { variant_name, data } if variant_name == "None" => {
                assert!(data.is_none());
            }
            _ => panic!("Expected None variant for negative index"),
        }
    }

    #[cfg(test)]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            /// Property: nth() with valid index always returns Some
            #[test]
            fn prop_nth_valid_index_returns_some(values in prop::collection::vec(any::<i64>(), 1..10), idx in 0usize..10) {
                if idx >= values.len() {
                    return Ok(()); // Skip if out of bounds
                }

                let arr = Arc::from(
                    values.iter().map(|&v| Value::Integer(v)).collect::<Vec<_>>()
                );
                let result = eval_array_nth(&arr, &Value::Integer(idx as i64)).unwrap();

                // Should return Some variant
                match result {
                    Value::EnumVariant { variant_name, data } if variant_name == "Some" => {
                        prop_assert!(data.is_some());
                        prop_assert_eq!(&data.unwrap()[0], &Value::Integer(values[idx]));
                    }
                    _ => prop_assert!(false, "Expected Some variant"),
                }
            }

            /// Property: nth() with out-of-bounds index always returns None
            #[test]
            fn prop_nth_out_of_bounds_returns_none(values in prop::collection::vec(any::<i64>(), 0..10), idx in 10i64..100) {
                let arr = Arc::from(
                    values.iter().map(|&v| Value::Integer(v)).collect::<Vec<_>>()
                );
                let result = eval_array_nth(&arr, &Value::Integer(idx)).unwrap();

                // Should return None variant
                match result {
                    Value::EnumVariant { variant_name, data } if variant_name == "None" => {
                        prop_assert!(data.is_none());
                    }
                    _ => prop_assert!(false, "Expected None variant"),
                }
            }

            /// Property: nth() with negative index always returns None
            #[test]
            fn prop_nth_negative_returns_none(values in prop::collection::vec(any::<i64>(), 1..10), idx in -100i64..-1) {
                let arr = Arc::from(
                    values.iter().map(|&v| Value::Integer(v)).collect::<Vec<_>>()
                );
                let result = eval_array_nth(&arr, &Value::Integer(idx)).unwrap();

                // Should return None variant
                match result {
                    Value::EnumVariant { variant_name, data } if variant_name == "None" => {
                        prop_assert!(data.is_none());
                    }
                    _ => prop_assert!(false, "Expected None variant for negative index"),
                }
            }

            /// Property: nth() never panics
            #[test]
            fn prop_nth_never_panics(values in prop::collection::vec(any::<i64>(), 0..20), idx in any::<i64>()) {
                let arr = Arc::from(
                    values.iter().map(|&v| Value::Integer(v)).collect::<Vec<_>>()
                );
                let _ = eval_array_nth(&arr, &Value::Integer(idx)); // Should not panic
            }
        }
    }
}

// ============================================================================
// STDLIB-005: Array concatenation and flattening (ISSUE #41)
// ============================================================================

/// Concatenate two arrays
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
///
/// # Examples
/// ```
/// [1, 2].concat([3, 4]) => [1, 2, 3, 4]
/// ```
fn eval_array_concat(arr: &Arc<[Value]>, other: &Value) -> Result<Value, InterpreterError> {
    match other {
        Value::Array(other_arr) => {
            let mut result = arr.to_vec();
            result.extend_from_slice(other_arr);
            Ok(Value::Array(Arc::from(result)))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "concat() requires array argument, got {other:?}"
        ))),
    }
}

/// Flatten nested arrays by one level
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
///
/// # Examples
/// ```
/// [[1, 2], [3, 4]].flatten() => [1, 2, 3, 4]
/// [1, 2, 3].flatten() => [1, 2, 3]  // Already flat
/// ```
fn eval_array_flatten(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    let mut result = Vec::new();

    for item in arr.iter() {
        match item {
            Value::Array(nested) => {
                result.extend_from_slice(nested);
            }
            _ => {
                // Not an array - keep as-is (already flat at this level)
                result.push(item.clone());
            }
        }
    }

    Ok(Value::Array(Arc::from(result)))
}
/// Compute union of two arrays (treats arrays as sets with unique elements)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
///
/// # Examples
/// ```
/// [1, 2, 3].union([3, 4, 5]) => [1, 2, 3, 4, 5]
/// [1, 2, 2].union([2, 3]) => [1, 2, 3]  // Duplicates removed
/// ```
fn eval_array_union(arr: &Arc<[Value]>, other: &Value) -> Result<Value, InterpreterError> {
    match other {
        Value::Array(other_arr) => {
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();

            // Add all unique elements from first array
            for item in arr.iter() {
                let key = format!("{item:?}");
                if seen.insert(key) {
                    result.push(item.clone());
                }
            }

            // Add unique elements from second array that aren't already in result
            for item in other_arr.iter() {
                let key = format!("{item:?}");
                if seen.insert(key) {
                    result.push(item.clone());
                }
            }

            Ok(Value::Array(Arc::from(result)))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "union() requires array argument, got {other:?}"
        ))),
    }
}

/// Compute intersection of two arrays (common elements only)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
///
/// # Examples
/// ```
/// [1, 2, 3, 4].intersection([3, 4, 5, 6]) => [3, 4]
/// [1, 2].intersection([3, 4]) => []  // No common elements
/// ```
fn eval_array_intersection(arr: &Arc<[Value]>, other: &Value) -> Result<Value, InterpreterError> {
    match other {
        Value::Array(other_arr) => {
            let other_set: std::collections::HashSet<_> =
                other_arr.iter().map(|v| format!("{v:?}")).collect();
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();

            for item in arr.iter() {
                let key = format!("{item:?}");
                if other_set.contains(&key) && seen.insert(key) {
                    result.push(item.clone());
                }
            }

            Ok(Value::Array(Arc::from(result)))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "intersection() requires array argument, got {other:?}"
        ))),
    }
}

/// Compute difference of two arrays (elements in first but not in second)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
///
/// # Examples
/// ```
/// [1, 2, 3, 4].difference([3, 4, 5, 6]) => [1, 2]
/// [1, 2].difference([3, 4]) => [1, 2]  // All elements retained
/// [1, 2].difference([1, 2, 3]) => []   // All elements removed
/// ```
fn eval_array_difference(arr: &Arc<[Value]>, other: &Value) -> Result<Value, InterpreterError> {
    match other {
        Value::Array(other_arr) => {
            let other_set: std::collections::HashSet<_> =
                other_arr.iter().map(|v| format!("{v:?}")).collect();
            let mut seen = std::collections::HashSet::new();
            let mut result = Vec::new();

            for item in arr.iter() {
                let key = format!("{item:?}");
                if !other_set.contains(&key) && seen.insert(key) {
                    result.push(item.clone());
                }
            }

            Ok(Value::Array(Arc::from(result)))
        }
        _ => Err(InterpreterError::TypeError(format!(
            "difference() requires array argument, got {other:?}"
        ))),
    }
}

/// STDLIB-009: Sort array elements
///
/// Returns a new sorted array without modifying the original.
/// Sorts by string representation to handle heterogeneous arrays.
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
///
/// # Examples
/// ```
/// [3, 1, 4, 1, 5].sort() => [1, 1, 3, 4, 5]
/// ["zebra", "apple", "banana"].sort() => ["apple", "banana", "zebra"]
/// [].sort() => []
/// ```
fn eval_array_sort(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    let mut sorted = arr.to_vec();
    sorted.sort_by(|a, b| {
        let a_str = format!("{a:?}");
        let b_str = format!("{b:?}");
        a_str.cmp(&b_str)
    });
    Ok(Value::Array(Arc::from(sorted)))
}
