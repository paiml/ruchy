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
fn eval_array_nullary_method(
    arr: &Arc<[Value]>,
    method: &str,
) -> Option<Result<Value, InterpreterError>> {
    let result = match method {
        "len" | "length" => eval_array_len(arr),
        "first" => eval_array_first(arr),
        "last" => eval_array_last(arr),
        "is_empty" => eval_array_is_empty(arr),
        "pop" => eval_array_pop(arr),
        "unique" => eval_array_unique(arr),
        "enumerate" => eval_array_enumerate(arr),
        "flatten" => eval_array_flatten(arr),
        "sort" => eval_array_sort(arr),
        "reverse" => eval_array_reverse(arr),
        "sum" => eval_array_sum(arr),
        "product" => eval_array_product(arr),
        "min" => eval_array_min(arr),
        "max" => eval_array_max(arr),
        _ => return None,
    };
    Some(result)
}

fn eval_array_unary_method(
    arr: &Arc<[Value]>,
    method: &str,
    arg: &Value,
) -> Option<Result<Value, InterpreterError>> {
    let result = match method {
        "push" => eval_array_push(arr, arg),
        "get" => eval_array_get(arr, arg),
        "nth" => eval_array_nth(arr, arg),
        "contains" => eval_array_contains(arr, arg),
        "join" => eval_array_join(arr, arg),
        "concat" | "append" => eval_array_concat(arr, arg),
        "union" => eval_array_union(arr, arg),
        "intersection" => eval_array_intersection(arr, arg),
        "difference" => eval_array_difference(arr, arg),
        "take" => eval_array_take(arr, arg),
        "skip" => eval_array_skip(arr, arg),
        "zip" => eval_array_zip(arr, arg),
        _ => return None,
    };
    Some(result)
}

fn eval_array_simple_method(
    arr: &Arc<[Value]>,
    method: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    if args.is_empty() {
        if let Some(result) = eval_array_nullary_method(arr, method) {
            return result.map(Some);
        }
    }
    if args.len() == 1 {
        if let Some(result) = eval_array_unary_method(arr, method, &args[0]) {
            return result.map(Some);
        }
    }
    if args.len() == 2 && method == "slice" {
        return eval_array_slice(arr, &args[0], &args[1]).map(Some);
    }
    Ok(None)
}

pub fn eval_array_method<F>(
    arr: &Arc<[Value]>,
    method: &str,
    args: &[Value],
    mut eval_function_call_value: F,
) -> Result<Value, InterpreterError>
where
    F: FnMut(&Value, &[Value]) -> Result<Value, InterpreterError>,
{
    if let Some(result) = eval_array_simple_method(arr, method, args)? {
        return Ok(result);
    }

    match method {
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
                enum_name: "Option".to_string(),
                variant_name: "None".to_string(),
                data: None,
            });
        }
        #[allow(clippy::cast_sign_loss)]
        let index = *idx as usize;
        if index < arr.len() {
            Ok(Value::EnumVariant {
                enum_name: "Option".to_string(),
                variant_name: "Some".to_string(),
                data: Some(vec![arr[index].clone()]),
            })
        } else {
            Ok(Value::EnumVariant {
                enum_name: "Option".to_string(),
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
        .map(|(i, val)| Value::Tuple(Arc::from(vec![Value::Integer(i as i64), val.clone()])))
        .collect();
    Ok(Value::Array(Arc::from(enumerated)))
}

// STDLIB-004: Custom array methods (complexity <= 5 each)

/// Extract slice from array
/// Complexity: 5 (within Toyota Way limits)
fn eval_array_slice(
    arr: &Arc<[Value]>,
    start: &Value,
    end: &Value,
) -> Result<Value, InterpreterError> {
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
    // Support both orderings: reduce(init, func) or reduce(func, init)
    let (initial, func) = if matches!(&args[1], Value::Closure { .. }) {
        // reduce(init, func) - Rust-like fold syntax
        (&args[0], &args[1])
    } else if matches!(&args[0], Value::Closure { .. }) {
        // reduce(func, init) - JavaScript-like syntax
        (&args[1], &args[0])
    } else {
        return Err(InterpreterError::RuntimeError(
            "reduce expects an initial value and a function".to_string(),
        ));
    };

    let mut accumulator = initial.clone();
    for item in arr.iter() {
        accumulator = eval_function_call_value(func, &[accumulator, item.clone()])?;
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

/// STDLIB-010: `Array.each()` method
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

/// PIPELINE-001: Reverse array order
/// Enables: arr |> reverse or `arr.reverse()`
fn eval_array_reverse(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    let mut reversed = arr.to_vec();
    reversed.reverse();
    Ok(Value::Array(Arc::from(reversed)))
}

/// BOOK-200: Sum all numeric elements in array
/// Enables: [1, 2, 3] |> `sum()` => 6
fn eval_array_sum(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    let mut int_sum: i64 = 0;
    let mut float_sum: f64 = 0.0;
    let mut has_float = false;

    for item in arr.iter() {
        match item {
            Value::Integer(i) => int_sum += i,
            Value::Float(f) => {
                has_float = true;
                float_sum += f;
            }
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "sum() requires numeric array elements".to_string(),
                ))
            }
        }
    }

    if has_float {
        Ok(Value::Float(int_sum as f64 + float_sum))
    } else {
        Ok(Value::Integer(int_sum))
    }
}

/// BOOK-200: Compute product of all numeric elements
/// Enables: [1, 2, 3] |> `product()` => 6
fn eval_array_product(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    if arr.is_empty() {
        return Ok(Value::Integer(1)); // Identity for multiplication
    }

    let mut int_product: i64 = 1;
    let mut float_product: f64 = 1.0;
    let mut has_float = false;

    for item in arr.iter() {
        match item {
            Value::Integer(i) => int_product *= i,
            Value::Float(f) => {
                has_float = true;
                float_product *= f;
            }
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "product() requires numeric array elements".to_string(),
                ))
            }
        }
    }

    if has_float {
        Ok(Value::Float(int_product as f64 * float_product))
    } else {
        Ok(Value::Integer(int_product))
    }
}

/// BOOK-200: Find minimum numeric element
/// Enables: [3, 1, 4] |> `min()` => 1
fn eval_array_min(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    if arr.is_empty() {
        return Ok(Value::Nil);
    }

    let mut min_val: Option<f64> = None;
    let mut has_float = false;

    for item in arr.iter() {
        let val = match item {
            Value::Integer(i) => *i as f64,
            Value::Float(f) => {
                has_float = true;
                *f
            }
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "min() requires numeric array elements".to_string(),
                ))
            }
        };
        min_val = Some(min_val.map_or(val, |m| m.min(val)));
    }

    match min_val {
        Some(v) if has_float => Ok(Value::Float(v)),
        Some(v) => Ok(Value::Integer(v as i64)),
        None => Ok(Value::Nil),
    }
}

/// BOOK-200: Find maximum numeric element
/// Enables: [3, 1, 4] |> `max()` => 4
fn eval_array_max(arr: &Arc<[Value]>) -> Result<Value, InterpreterError> {
    if arr.is_empty() {
        return Ok(Value::Nil);
    }

    let mut max_val: Option<f64> = None;
    let mut has_float = false;

    for item in arr.iter() {
        let val = match item {
            Value::Integer(i) => *i as f64,
            Value::Float(f) => {
                has_float = true;
                *f
            }
            _ => {
                return Err(InterpreterError::RuntimeError(
                    "max() requires numeric array elements".to_string(),
                ))
            }
        };
        max_val = Some(max_val.map_or(val, |m| m.max(val)));
    }

    match max_val {
        Some(v) if has_float => Ok(Value::Float(v)),
        Some(v) => Ok(Value::Integer(v as i64)),
        None => Ok(Value::Nil),
    }
}

/// BOOK-200: Take first n elements from array
/// Enables: [1, 2, 3, 4, 5].take(3) => [1, 2, 3]
fn eval_array_take(arr: &Arc<[Value]>, count: &Value) -> Result<Value, InterpreterError> {
    match count {
        Value::Integer(n) => {
            let n = (*n).max(0) as usize;
            let taken: Vec<Value> = arr.iter().take(n).cloned().collect();
            Ok(Value::Array(Arc::from(taken)))
        }
        _ => Err(InterpreterError::RuntimeError(
            "take() expects integer argument".to_string(),
        )),
    }
}

/// BOOK-200: Skip first n elements from array
/// Enables: [1, 2, 3, 4, 5].skip(2) => [3, 4, 5]
fn eval_array_skip(arr: &Arc<[Value]>, count: &Value) -> Result<Value, InterpreterError> {
    match count {
        Value::Integer(n) => {
            let n = (*n).max(0) as usize;
            let skipped: Vec<Value> = arr.iter().skip(n).cloned().collect();
            Ok(Value::Array(Arc::from(skipped)))
        }
        _ => Err(InterpreterError::RuntimeError(
            "skip() expects integer argument".to_string(),
        )),
    }
}

/// BOOK-200: Zip two arrays together into array of tuples
/// Enables: `[1, 2].zip(["a", "b"]) => [(1, "a"), (2, "b")]`
fn eval_array_zip(arr: &Arc<[Value]>, other: &Value) -> Result<Value, InterpreterError> {
    match other {
        Value::Array(other_arr) => {
            let zipped: Vec<Value> = arr
                .iter()
                .zip(other_arr.iter())
                .map(|(a, b)| Value::Tuple(Arc::from(vec![a.clone(), b.clone()])))
                .collect();
            Ok(Value::Array(Arc::from(zipped)))
        }
        _ => Err(InterpreterError::RuntimeError(
            "zip() expects array argument".to_string(),
        )),
    }
}


#[cfg(test)]
#[path = "eval_array_tests.rs"]
mod tests;
