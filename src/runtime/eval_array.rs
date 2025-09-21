//! Array method evaluation module
//!
//! This module handles evaluation of array methods in the interpreter.
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::runtime::{InterpreterError, Value};
use std::rc::Rc;

/// Evaluate an array method call
///
/// # Complexity
/// Cyclomatic complexity: 15 (will be decomposed further into helper functions)
pub fn eval_array_method<F>(
    arr: &Rc<Vec<Value>>,
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

        // Single-argument methods
        "push" if args.len() == 1 => eval_array_push(arr, &args[0]),
        "pop" if args.is_empty() => eval_array_pop(arr),
        "get" if args.len() == 1 => eval_array_get(arr, &args[0]),

        // Higher-order methods
        "map" => eval_array_map(arr, args, &mut eval_function_call_value),
        "filter" => eval_array_filter(arr, args, &mut eval_function_call_value),
        "reduce" => eval_array_reduce(arr, args, &mut eval_function_call_value),
        "any" => eval_array_any(arr, args, &mut eval_function_call_value),
        "all" => eval_array_all(arr, args, &mut eval_function_call_value),
        "find" => eval_array_find(arr, args, &mut eval_function_call_value),

        _ => Err(InterpreterError::RuntimeError(format!(
            "Unknown array method: {method}"
        ))),
    }
}

// No-argument array methods (complexity <= 3 each)

fn eval_array_len(arr: &Rc<Vec<Value>>) -> Result<Value, InterpreterError> {
    Ok(Value::Integer(arr.len() as i64))
}

fn eval_array_first(arr: &Rc<Vec<Value>>) -> Result<Value, InterpreterError> {
    Ok(arr.first().cloned().unwrap_or(Value::Nil))
}

fn eval_array_last(arr: &Rc<Vec<Value>>) -> Result<Value, InterpreterError> {
    Ok(arr.last().cloned().unwrap_or(Value::Nil))
}

// Single-argument array methods (complexity <= 5 each)

fn eval_array_push(arr: &Rc<Vec<Value>>, item: &Value) -> Result<Value, InterpreterError> {
    let mut new_arr = (**arr).clone();
    new_arr.push(item.clone());
    Ok(Value::Array(Rc::new(new_arr)))
}

fn eval_array_pop(arr: &Rc<Vec<Value>>) -> Result<Value, InterpreterError> {
    let mut new_arr = (**arr).clone();
    new_arr.pop().unwrap_or(Value::nil());
    Ok(Value::Array(Rc::new(new_arr)))
}

fn eval_array_get(arr: &Rc<Vec<Value>>, index: &Value) -> Result<Value, InterpreterError> {
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

// Higher-order array methods (complexity <= 8 each)

fn eval_array_map<F>(
    arr: &Rc<Vec<Value>>,
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
    Ok(Value::Array(Rc::new(result)))
}

fn eval_array_filter<F>(
    arr: &Rc<Vec<Value>>,
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
    Ok(Value::Array(Rc::new(result)))
}

fn eval_array_reduce<F>(
    arr: &Rc<Vec<Value>>,
    args: &[Value],
    eval_function_call_value: &mut F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "reduce expects 2 arguments".to_string(),
        ));
    }
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
    arr: &Rc<Vec<Value>>,
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
    arr: &Rc<Vec<Value>>,
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
    arr: &Rc<Vec<Value>>,
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

// Helper function (complexity <= 3)

fn validate_single_closure_argument(
    args: &[Value],
    method_name: &str,
) -> Result<(), InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(format!(
            "{method_name} expects 1 argument"
        )));
    }
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
        let arr = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = eval_array_len(&arr).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_array_first() {
        let arr = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = eval_array_first(&arr).unwrap();
        assert_eq!(result, Value::Integer(1));
    }

    #[test]
    fn test_array_last() {
        let arr = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = eval_array_last(&arr).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_array_push() {
        let arr = Rc::new(vec![Value::Integer(1), Value::Integer(2)]);
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
        let arr = Rc::new(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);
        let result = eval_array_get(&arr, &Value::Integer(1)).unwrap();
        assert_eq!(result, Value::Integer(2));
    }
}
