//! Builtin function evaluation module
//!
//! This module handles all builtin functions including math operations,
//! I/O functions, utility functions, and type operations.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::runtime::validation::validate_arg_count;
use crate::runtime::{InterpreterError, Value};

use std::collections::HashMap;
use std::sync::Arc;

/// Evaluate a builtin function call
///
/// Evaluate built-in function (complexity: 8, refactored from 11)
pub fn eval_builtin_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    // Dispatch to category-specific handlers (try each in sequence)
    let dispatchers: &[fn(&str, &[Value]) -> Result<Option<Value>, InterpreterError>] = &[
        try_eval_io_function,
        try_eval_math_function,
        try_eval_utility_function,
        try_eval_time_function,
        try_eval_dataframe_function,
        try_eval_environment_function,
        try_eval_fs_function,
        try_eval_path_function,
        try_eval_json_function,
        try_eval_http_function,
    ];

    for try_eval in dispatchers {
        if let Some(result) = try_eval(name, args)? {
            return Ok(Some(result));
        }
    }

    Ok(None)
}

fn try_eval_io_function(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_println__" => Ok(Some(eval_println(args)?)),
        "__builtin_print__" => Ok(Some(eval_print(args)?)),
        "__builtin_dbg__" => Ok(Some(eval_dbg(args)?)),
        _ => Ok(None),
    }
}

/// Try to evaluate math functions (complexity: 7, refactored from 13)
fn try_eval_math_function(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    // Try basic math functions first
    if let Some(result) = try_eval_basic_math(name, args)? {
        return Ok(Some(result));
    }

    // Try advanced math functions
    try_eval_advanced_math(name, args)
}

/// Basic math functions (sqrt, pow, abs, min, max)
/// Basic math functions - Part 1
/// Complexity: 4 (within Toyota Way limits)
fn try_eval_basic_math_part1(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_sqrt__" => Ok(Some(eval_sqrt(args)?)),
        "__builtin_pow__" => Ok(Some(eval_pow(args)?)),
        "__builtin_abs__" => Ok(Some(eval_abs(args)?)),
        _ => Ok(None),
    }
}

/// Basic math functions - Part 2
/// Complexity: 3 (within Toyota Way limits)
fn try_eval_basic_math_part2(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_min__" => Ok(Some(eval_min(args)?)),
        "__builtin_max__" => Ok(Some(eval_max(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for basic math functions
/// Complexity: 3 (within Toyota Way limits)
fn try_eval_basic_math(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    if let Some(result) = try_eval_basic_math_part1(name, args)? {
        return Ok(Some(result));
    }
    try_eval_basic_math_part2(name, args)
}

/// Advanced math functions - Part 1 (rounding)
/// Complexity: 4 (within Toyota Way limits)
fn try_eval_advanced_math_part1(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_floor__" => Ok(Some(eval_floor(args)?)),
        "__builtin_ceil__" => Ok(Some(eval_ceil(args)?)),
        "__builtin_round__" => Ok(Some(eval_round(args)?)),
        _ => Ok(None),
    }
}

/// Advanced math functions - Part 2 (trigonometry)
/// Complexity: 4 (within Toyota Way limits)
fn try_eval_advanced_math_part2(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_sin__" => Ok(Some(eval_sin(args)?)),
        "__builtin_cos__" => Ok(Some(eval_cos(args)?)),
        "__builtin_tan__" => Ok(Some(eval_tan(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for advanced math functions
/// Complexity: 3 (within Toyota Way limits)
fn try_eval_advanced_math(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    if let Some(result) = try_eval_advanced_math_part1(name, args)? {
        return Ok(Some(result));
    }
    try_eval_advanced_math_part2(name, args)
}

/// Utility functions - Part 1
/// Complexity: 3 (within Toyota Way limits)
fn try_eval_utility_part1(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_len__" => Ok(Some(eval_len(args)?)),
        "__builtin_range__" => Ok(Some(eval_range(args)?)),
        _ => Ok(None),
    }
}

/// Utility functions - Part 2
/// Complexity: 3 (within Toyota Way limits)
fn try_eval_utility_part2(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_type__" => Ok(Some(eval_type(args)?)),
        "__builtin_reverse__" => Ok(Some(eval_reverse(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for utility functions
/// Complexity: 3 (within Toyota Way limits, reduced from 6)
fn try_eval_utility_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    if let Some(result) = try_eval_utility_part1(name, args)? {
        return Ok(Some(result));
    }
    try_eval_utility_part2(name, args)
}

fn try_eval_time_function(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_sleep__" => Ok(Some(eval_sleep(args)?)),
        "__builtin_timestamp__" => Ok(Some(eval_timestamp(args)?)),
        _ => Ok(None),
    }
}

fn try_eval_dataframe_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_dataframe_new__" => Ok(Some(eval_dataframe_new(args)?)),
        "__builtin_dataframe_from_csv_string__" => Ok(Some(eval_dataframe_from_csv_string(args)?)),
        "__builtin_dataframe_from_json__" => Ok(Some(eval_dataframe_from_json(args)?)),
        _ => Ok(None),
    }
}

/// Print values to stdout with newline
///
/// Supports printf-style formatting with {} placeholders:
/// - `println("Count: {}", 42)` → "Count: 42"
/// - `println("Name: {}, Age: {}", "Alice", 30)` → "Name: Alice, Age: 30"
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
/// Format string with interpolation
/// Complexity: 2 (within Toyota Way limits)
fn format_with_interpolation(fmt_str: &str, args: &[Value]) -> String {
    let mut result = fmt_str.to_string();
    for arg in args {
        if let Some(pos) = result.find("{}") {
            result.replace_range(pos..pos + 2, &format!("{arg}"));
        }
    }
    result
}

/// Join values with spaces
/// Complexity: 1 (within Toyota Way limits)
fn join_values(args: &[Value]) -> String {
    args.iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Format println output
/// Complexity: 3 (within Toyota Way limits, reduced from 7)
fn format_println_output(args: &[Value]) -> String {
    if args.is_empty() {
        "\n".to_string()
    } else if let Value::String(fmt_str) = &args[0] {
        if fmt_str.contains("{}") {
            format!("{}\n", format_with_interpolation(fmt_str, &args[1..]))
        } else {
            format!("{}\n", join_values(args))
        }
    } else {
        format!("{}\n", join_values(args))
    }
}

/// Print values to stdout with newline
/// Complexity: 2 (within Toyota Way limits, reduced from 7)
fn eval_println(args: &[Value]) -> Result<Value, InterpreterError> {
    let output = format_println_output(args);

    // Write to output buffer (for notebook capture)
    if let Ok(mut buf) = crate::runtime::builtins::OUTPUT_BUFFER.lock() {
        buf.push_str(&output);
    }

    // Also write to stdout for local REPL use
    print!("{output}");

    Ok(Value::Nil)
}

/// Print values to stdout without newline
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_print(args: &[Value]) -> Result<Value, InterpreterError> {
    let output = args
        .iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(" ");

    // Write to output buffer (for notebook capture)
    if let Ok(mut buf) = crate::runtime::builtins::OUTPUT_BUFFER.lock() {
        buf.push_str(&output);
    }

    // Also write to stdout for local REPL use
    print!("{output}");

    Ok(Value::Nil)
}

/// Debug print with value inspection
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_dbg(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() == 1 {
        println!("[DEBUG] {:?}", args[0]);
        Ok(args[0].clone())
    } else {
        println!("[DEBUG] {args:?}");
        Ok(Value::from_array(args.to_vec()))
    }
}

/// Square root function
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits, reduced from 4)
fn eval_sqrt(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("sqrt", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).sqrt())),
        Value::Float(f) => Ok(Value::Float(f.sqrt())),
        _ => Err(InterpreterError::RuntimeError(
            "sqrt() expects a number".to_string(),
        )),
    }
}

/// Power function (base^exponent)
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits, reduced from 7)
fn eval_pow(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("pow", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::Integer(base), Value::Integer(exp)) => {
            if *exp >= 0 {
                Ok(Value::Integer(base.pow(*exp as u32)))
            } else {
                Ok(Value::Float((*base as f64).powf(*exp as f64)))
            }
        }
        (Value::Float(base), Value::Integer(exp)) => Ok(Value::Float(base.powf(*exp as f64))),
        (Value::Integer(base), Value::Float(exp)) => Ok(Value::Float((*base as f64).powf(*exp))),
        (Value::Float(base), Value::Float(exp)) => Ok(Value::Float(base.powf(*exp))),
        _ => Err(InterpreterError::RuntimeError(
            "pow() expects two numbers".to_string(),
        )),
    }
}

/// Absolute value function
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits, reduced from 4)
fn eval_abs(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("abs", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(InterpreterError::RuntimeError(
            "abs() expects a number".to_string(),
        )),
    }
}

/// Minimum of two values
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits, reduced from 6)
fn eval_min(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("min", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a.min(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).min(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.min(*b as f64))),
        _ => Err(InterpreterError::RuntimeError(
            "min() expects two numbers".to_string(),
        )),
    }
}

/// Maximum of two values
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits, reduced from 6)
fn eval_max(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("max", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(*a.max(b))),
        (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(*b))),
        (Value::Integer(a), Value::Float(b)) => Ok(Value::Float((*a as f64).max(*b))),
        (Value::Float(a), Value::Integer(b)) => Ok(Value::Float(a.max(*b as f64))),
        _ => Err(InterpreterError::RuntimeError(
            "max() expects two numbers".to_string(),
        )),
    }
}

/// Floor function (round down)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits, reduced from 4)
fn eval_floor(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("floor", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(f.floor() as i64)),
        _ => Err(InterpreterError::RuntimeError(
            "floor() expects a number".to_string(),
        )),
    }
}

/// Ceiling function (round up)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits, reduced from 4)
fn eval_ceil(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("ceil", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(f.ceil() as i64)),
        _ => Err(InterpreterError::RuntimeError(
            "ceil() expects a number".to_string(),
        )),
    }
}

/// Round to nearest integer
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits, reduced from 4)
fn eval_round(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("round", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(f.round() as i64)),
        _ => Err(InterpreterError::RuntimeError(
            "round() expects a number".to_string(),
        )),
    }
}

/// Sine function
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits, reduced from 4)
fn eval_sin(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("sin", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).sin())),
        Value::Float(f) => Ok(Value::Float(f.sin())),
        _ => Err(InterpreterError::RuntimeError(
            "sin() expects a number".to_string(),
        )),
    }
}

/// Cosine function
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits, reduced from 4)
fn eval_cos(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("cos", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).cos())),
        Value::Float(f) => Ok(Value::Float(f.cos())),
        _ => Err(InterpreterError::RuntimeError(
            "cos() expects a number".to_string(),
        )),
    }
}

/// Tangent function
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits, reduced from 4)
fn eval_tan(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("tan", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).tan())),
        Value::Float(f) => Ok(Value::Float(f.tan())),
        _ => Err(InterpreterError::RuntimeError(
            "tan() expects a number".to_string(),
        )),
    }
}

/// Length of collections and strings
///
/// # Complexity
/// Cyclomatic complexity: 5 (within Toyota Way limits, reduced from 6)
fn eval_len(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("len", args, 1)?;
    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        Value::DataFrame { columns } => {
            if columns.is_empty() {
                Ok(Value::Integer(0))
            } else {
                Ok(Value::Integer(columns[0].values.len() as i64))
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "len() expects a string, array, or dataframe".to_string(),
        )),
    }
}

/// Generate ranges of integers
///
/// # Complexity
/// Cyclomatic complexity: 9 (within Toyota Way limits)
fn eval_range(args: &[Value]) -> Result<Value, InterpreterError> {
    match args.len() {
        1 => eval_range_one_arg(&args[0]),
        2 => eval_range_two_args(&args[0], &args[1]),
        3 => eval_range_three_args(&args[0], &args[1], &args[2]),
        _ => Err(InterpreterError::RuntimeError(
            "range() expects 1, 2, or 3 arguments".to_string(),
        )),
    }
}

/// Range with single argument: range(end) -> 0..end
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_range_one_arg(end_val: &Value) -> Result<Value, InterpreterError> {
    match end_val {
        Value::Integer(end) => {
            let mut result = Vec::new();
            for i in 0..*end {
                result.push(Value::Integer(i));
            }
            Ok(Value::Array(result.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "range() expects integer arguments".to_string(),
        )),
    }
}

/// Range with two arguments: range(start, end) -> start..end
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_range_two_args(start_val: &Value, end_val: &Value) -> Result<Value, InterpreterError> {
    match (start_val, end_val) {
        (Value::Integer(start), Value::Integer(end)) => {
            let mut result = Vec::new();
            for i in *start..*end {
                result.push(Value::Integer(i));
            }
            Ok(Value::Array(result.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "range() expects integer arguments".to_string(),
        )),
    }
}

/// Range with three arguments: range(start, end, step) -> start..end by step
///
/// # Complexity
/// Cyclomatic complexity: 8 (within Toyota Way limits)
/// Generate range with positive step
/// Complexity: 2 (within Toyota Way limits)
fn generate_range_forward(start: i64, end: i64, step: i64) -> Vec<Value> {
    let mut result = Vec::new();
    let mut i = start;
    while i < end {
        result.push(Value::Integer(i));
        i += step;
    }
    result
}

/// Generate range with negative step
/// Complexity: 2 (within Toyota Way limits)
fn generate_range_backward(start: i64, end: i64, step: i64) -> Vec<Value> {
    let mut result = Vec::new();
    let mut i = start;
    while i > end {
        result.push(Value::Integer(i));
        i += step;
    }
    result
}

/// Range function with three arguments (start, end, step)
/// Complexity: 4 (within Toyota Way limits, reduced from 6)
fn eval_range_three_args(
    start_val: &Value,
    end_val: &Value,
    step_val: &Value,
) -> Result<Value, InterpreterError> {
    match (start_val, end_val, step_val) {
        (Value::Integer(start), Value::Integer(end), Value::Integer(step)) => {
            if *step == 0 {
                return Err(InterpreterError::RuntimeError(
                    "range() step cannot be zero".to_string(),
                ));
            }
            let result = if *step > 0 {
                generate_range_forward(*start, *end, *step)
            } else {
                generate_range_backward(*start, *end, *step)
            };
            Ok(Value::Array(result.into()))
        }
        _ => Err(InterpreterError::RuntimeError(
            "range() expects integer arguments".to_string(),
        )),
    }
}

/// Get type name of a value
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits, reduced from 3)
fn eval_type(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("type", args, 1)?;
    Ok(Value::from_string(args[0].type_name().to_string()))
}

/// Reverse arrays and strings
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits, reduced from 5)
fn eval_reverse(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("reverse", args, 1)?;
    match &args[0] {
        Value::Array(arr) => {
            let mut reversed = arr.to_vec();
            reversed.reverse();
            Ok(Value::from_array(reversed))
        }
        Value::String(s) => {
            let reversed: String = s.chars().rev().collect();
            Ok(Value::from_string(reversed))
        }
        _ => Err(InterpreterError::RuntimeError(
            "reverse() expects an array or string".to_string(),
        )),
    }
}

/// Sleep for a duration in milliseconds
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits, reduced from 3)
fn eval_sleep(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("sleep", args, 1)?;

    let millis = match &args[0] {
        Value::Integer(n) => *n as u64,
        Value::Float(f) => *f as u64,
        _ => {
            return Err(InterpreterError::RuntimeError(
                "sleep() expects a numeric argument".to_string(),
            ))
        }
    };

    std::thread::sleep(std::time::Duration::from_millis(millis));
    Ok(Value::Nil)
}

/// `timestamp()` - Get current time in milliseconds since Unix epoch
///
/// # Examples
/// ```
/// let start = timestamp();
/// // ... some operation ...
/// let end = timestamp();
/// let duration = end - start;
/// ```
///
/// # Complexity
/// Cyclomatic complexity: 2
fn eval_timestamp(args: &[Value]) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "timestamp() expects no arguments".to_string(),
        ));
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| InterpreterError::RuntimeError(format!("System time error: {e}")))?;

    Ok(Value::Integer(now.as_millis() as i64))
}

/// `DataFrame::new()` - Create a new `DataFrame` builder
///
/// Returns a builder object that accumulates columns via `.column()` calls
/// and finalizes with `.build()` to create the `DataFrame`.
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_dataframe_new(args: &[Value]) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "DataFrame::new() takes no arguments".to_string(),
        ));
    }

    // Create a builder object with:
    // - __type: "DataFrameBuilder" marker
    // - __columns: empty array to accumulate columns
    let mut builder = std::collections::HashMap::new();
    builder.insert(
        "__type".to_string(),
        Value::from_string("DataFrameBuilder".to_string()),
    );
    builder.insert("__columns".to_string(), Value::from_array(vec![]));

    Ok(Value::Object(std::sync::Arc::new(builder)))
}

/// `DataFrame::from_csv_string()` - Parse CSV data into `DataFrame`
/// Performs type inference for integers, floats, and strings
/// Complexity: 8 (within Toyota Way limits, reduced from 9)
fn eval_dataframe_from_csv_string(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("DataFrame::from_csv_string", args, 1)?;

    let csv_string = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame::from_csv_string() expects string argument".to_string(),
            ))
        }
    };

    parse_csv_to_dataframe(csv_string)
}

/// `DataFrame::from_json()` - Parse JSON array into `DataFrame`
/// Expects array of objects with consistent keys
/// Complexity: 7 (within Toyota Way limits, reduced from 8)
fn eval_dataframe_from_json(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("DataFrame::from_json", args, 1)?;

    let json_string = match &args[0] {
        Value::String(s) => s.as_ref(),
        _ => {
            return Err(InterpreterError::RuntimeError(
                "DataFrame::from_json() expects string argument".to_string(),
            ))
        }
    };

    parse_json_to_dataframe(json_string)
}

/// Parse CSV string into `DataFrame` with type inference
/// Complexity: 9 (within Toyota Way limits)
fn parse_csv_to_dataframe(csv: &str) -> Result<Value, InterpreterError> {
    let lines: Vec<&str> = csv.trim().lines().collect();

    if lines.is_empty() {
        return Ok(Value::DataFrame { columns: vec![] });
    }

    // Parse header
    let headers: Vec<String> = lines[0].split(',').map(|s| s.trim().to_string()).collect();

    // Initialize columns
    let mut columns: Vec<crate::runtime::DataFrameColumn> = headers
        .iter()
        .map(|name| crate::runtime::DataFrameColumn {
            name: name.clone(),
            values: Vec::new(),
        })
        .collect();

    // Parse data rows with type inference
    for line in lines.iter().skip(1) {
        let values: Vec<&str> = line.split(',').map(str::trim).collect();

        for (col_idx, value_str) in values.iter().enumerate() {
            if col_idx < columns.len() {
                let value = infer_value_type(value_str);
                columns[col_idx].values.push(value);
            }
        }
    }

    Ok(Value::DataFrame { columns })
}

/// Parse JSON array into `DataFrame`
/// Complexity: 5 (within Toyota Way limits - refactored from 14)
fn parse_json_to_dataframe(json_str: &str) -> Result<Value, InterpreterError> {
    let json_str = json_str.trim();

    if json_str == "[]" {
        return Ok(Value::DataFrame { columns: vec![] });
    }

    validate_json_array_format(json_str)?;

    let objects = extract_json_objects(json_str)?;

    if objects.is_empty() {
        return Ok(Value::DataFrame { columns: vec![] });
    }

    build_dataframe_from_objects(&objects)
}

/// Validate JSON array format
fn validate_json_array_format(json_str: &str) -> Result<(), InterpreterError> {
    if !json_str.starts_with('[') || !json_str.ends_with(']') {
        return Err(InterpreterError::RuntimeError(
            "DataFrame::from_json() expects JSON array".to_string(),
        ));
    }
    Ok(())
}

/// Build `DataFrame` from parsed JSON objects
fn build_dataframe_from_objects(objects: &[String]) -> Result<Value, InterpreterError> {
    use std::collections::HashMap;

    // Collect all column names from first object
    let column_names = extract_json_keys(&objects[0])?;

    // Initialize columns
    let mut columns_map: HashMap<String, Vec<Value>> = HashMap::new();
    for name in &column_names {
        columns_map.insert(name.clone(), Vec::new());
    }

    // Parse each object and populate columns
    populate_columns_from_objects(objects, &mut columns_map)?;

    // Convert to DataFrame columns
    convert_to_dataframe_columns(column_names, columns_map)
}

/// Populate columns from JSON objects
fn populate_columns_from_objects(
    objects: &[String],
    columns_map: &mut std::collections::HashMap<String, Vec<Value>>,
) -> Result<(), InterpreterError> {
    for obj_str in objects {
        let key_values = parse_json_object(obj_str)?;
        for (key, value) in key_values {
            if let Some(col_values) = columns_map.get_mut(&key) {
                col_values.push(value);
            }
        }
    }
    Ok(())
}

/// Convert column map to `DataFrame` columns
fn convert_to_dataframe_columns(
    column_names: Vec<String>,
    mut columns_map: std::collections::HashMap<String, Vec<Value>>,
) -> Result<Value, InterpreterError> {
    let mut columns = Vec::new();
    for name in column_names {
        if let Some(values) = columns_map.remove(&name) {
            columns.push(crate::runtime::DataFrameColumn { name, values });
        }
    }
    Ok(Value::DataFrame { columns })
}

/// Infer Value type from string (int, float, or string)
/// Complexity: 4 (within Toyota Way limits)
fn infer_value_type(s: &str) -> Value {
    // Try integer first
    if let Ok(i) = s.parse::<i64>() {
        return Value::Integer(i);
    }

    // Try float
    if let Ok(f) = s.parse::<f64>() {
        return Value::Float(f);
    }

    // Default to string
    Value::from_string(s.to_string())
}

/// Extract JSON objects from array string (complexity: 5, refactored from 11)
fn extract_json_objects(json_str: &str) -> Result<Vec<String>, InterpreterError> {
    let inner = &json_str[1..json_str.len() - 1].trim();

    if inner.is_empty() {
        return Ok(vec![]);
    }

    let mut objects = Vec::new();
    let mut state = JsonParserState::default();

    for ch in inner.chars() {
        process_json_char(ch, &mut state, &mut objects);
    }

    Ok(objects)
}

/// JSON parser state
#[derive(Default)]
struct JsonParserState {
    current: String,
    brace_count: i32,
    in_string: bool,
}

/// Process a single JSON character
/// Handle opening brace in JSON parsing
/// Complexity: 1 (within Toyota Way limits)
fn handle_opening_brace(state: &mut JsonParserState, ch: char) {
    state.brace_count += 1;
    state.current.push(ch);
}

/// Handle closing brace in JSON parsing
/// Complexity: 2 (within Toyota Way limits)
fn handle_closing_brace(state: &mut JsonParserState, ch: char, objects: &mut Vec<String>) {
    state.brace_count -= 1;
    state.current.push(ch);
    if state.brace_count == 0 {
        objects.push(state.current.trim().to_string());
        state.current.clear();
    }
}

/// Handle default character in JSON parsing
/// Complexity: 2 (within Toyota Way limits)
fn handle_json_default_char(ch: char, state: &mut JsonParserState) {
    if state.brace_count > 0 {
        state.current.push(ch);
    }
}

/// Process a single character in JSON parsing
/// Complexity: 5 (within Toyota Way limits, reduced from 7)
fn process_json_char(ch: char, state: &mut JsonParserState, objects: &mut Vec<String>) {
    match ch {
        '"' => state.in_string = !state.in_string,
        '{' if !state.in_string => handle_opening_brace(state, ch),
        '}' if !state.in_string => handle_closing_brace(state, ch, objects),
        ',' if !state.in_string && state.brace_count == 0 => {}
        _ => handle_json_default_char(ch, state),
    }
}

/// Extract keys from a JSON object string
/// Complexity: 5 (within Toyota Way limits)
fn extract_json_keys(obj_str: &str) -> Result<Vec<String>, InterpreterError> {
    let mut keys = Vec::new();
    let inner = obj_str.trim().trim_start_matches('{').trim_end_matches('}');

    for pair in inner.split(',') {
        if let Some(colon_pos) = pair.find(':') {
            let key = pair[..colon_pos].trim().trim_matches('"');
            keys.push(key.to_string());
        }
    }

    Ok(keys)
}

/// Parse JSON object into key-value pairs
/// Complexity: 7 (within Toyota Way limits)
/// Parse JSON value from string
/// Complexity: 4 (within Toyota Way limits)
fn parse_json_value(value_str: &str) -> Value {
    if value_str.starts_with('"') {
        // String value
        let unquoted = value_str.trim_matches('"');
        Value::from_string(unquoted.to_string())
    } else if let Ok(i) = value_str.parse::<i64>() {
        // Integer value
        Value::Integer(i)
    } else if let Ok(f) = value_str.parse::<f64>() {
        // Float value
        Value::Float(f)
    } else {
        // Default to string
        Value::from_string(value_str.to_string())
    }
}

/// Parse JSON object string into key-value pairs
/// Complexity: 3 (within Toyota Way limits, reduced from 6)
fn parse_json_object(obj_str: &str) -> Result<Vec<(String, Value)>, InterpreterError> {
    let mut pairs = Vec::new();
    let inner = obj_str.trim().trim_start_matches('{').trim_end_matches('}');

    for pair in inner.split(',') {
        if let Some(colon_pos) = pair.find(':') {
            let key = pair[..colon_pos].trim().trim_matches('"').to_string();
            let value_str = pair[colon_pos + 1..].trim();
            let value = parse_json_value(value_str);
            pairs.push((key, value));
        }
    }

    Ok(pairs)
}

// Environment Functions

/// Dispatch environment functions - Part 1
/// Complexity: 5 (within Toyota Way limits)
fn try_eval_env_part1(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_env_args__" => Ok(Some(eval_env_args(args)?)),
        "__builtin_env_var__" => Ok(Some(eval_env_var(args)?)),
        "__builtin_env_set_var__" => Ok(Some(eval_env_set_var(args)?)),
        "__builtin_env_remove_var__" => Ok(Some(eval_env_remove_var(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch environment functions - Part 2
/// Complexity: 5 (within Toyota Way limits)
fn try_eval_env_part2(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_env_vars__" => Ok(Some(eval_env_vars(args)?)),
        "__builtin_env_current_dir__" => Ok(Some(eval_env_current_dir(args)?)),
        "__builtin_env_set_current_dir__" => Ok(Some(eval_env_set_current_dir(args)?)),
        "__builtin_env_temp_dir__" => Ok(Some(eval_env_temp_dir(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for environment functions
/// Complexity: 3 (within Toyota Way limits, reduced from 10)
fn try_eval_environment_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    if let Some(result) = try_eval_env_part1(name, args)? {
        return Ok(Some(result));
    }
    try_eval_env_part2(name, args)
}

/// Evaluate env_args() builtin function
/// Returns command-line arguments as an array of strings
/// Complexity: 2 (within Toyota Way limits)
fn eval_env_args(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_args", args, 0)?;

    // Get command-line arguments
    let cmd_args: Vec<Value> = std::env::args()
        .map(|s| Value::from_string(s))
        .collect();

    Ok(Value::from_array(cmd_args))
}

/// Evaluate env_var() builtin function
/// Returns environment variable value by key
/// Complexity: 3 (within Toyota Way limits)
fn eval_env_var(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_var", args, 1)?;

    match &args[0] {
        Value::String(key) => match std::env::var(key.as_ref()) {
            Ok(val) => Ok(Value::from_string(val)),
            Err(_) => Err(InterpreterError::RuntimeError(
                format!("Environment variable '{}' not found", key),
            )),
        },
        _ => Err(InterpreterError::RuntimeError(
            "env_var() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate env_set_var() builtin function
/// Sets environment variable
/// Complexity: 3 (within Toyota Way limits)
fn eval_env_set_var(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_set_var", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(key), Value::String(value)) => {
            std::env::set_var(key.as_ref(), value.as_ref());
            Ok(Value::Nil)
        }
        _ => Err(InterpreterError::RuntimeError(
            "env_set_var() expects two string arguments".to_string(),
        )),
    }
}

/// Evaluate env_remove_var() builtin function
/// Removes environment variable
/// Complexity: 2 (within Toyota Way limits)
fn eval_env_remove_var(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_remove_var", args, 1)?;

    match &args[0] {
        Value::String(key) => {
            std::env::remove_var(key.as_ref());
            Ok(Value::Nil)
        }
        _ => Err(InterpreterError::RuntimeError(
            "env_remove_var() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate env_vars() builtin function
/// Returns all environment variables as HashMap
/// Complexity: 1 (within Toyota Way limits)
fn eval_env_vars(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_vars", args, 0)?;

    let vars: HashMap<String, Value> = std::env::vars()
        .map(|(k, v)| (k, Value::from_string(v)))
        .collect();

    Ok(Value::Object(Arc::new(vars)))
}

/// Evaluate env_current_dir() builtin function
/// Returns current working directory
/// Complexity: 2 (within Toyota Way limits)
fn eval_env_current_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_current_dir", args, 0)?;

    match std::env::current_dir() {
        Ok(path) => Ok(Value::from_string(path.to_string_lossy().to_string())),
        Err(e) => Err(InterpreterError::RuntimeError(
            format!("Failed to get current directory: {}", e),
        )),
    }
}

/// Evaluate env_set_current_dir() builtin function
/// Changes current working directory
/// Complexity: 2 (within Toyota Way limits)
fn eval_env_set_current_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_set_current_dir", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::env::set_current_dir(path.as_ref()) {
            Ok(_) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(
                format!("Failed to set current directory: {}", e),
            )),
        },
        _ => Err(InterpreterError::RuntimeError(
            "env_set_current_dir() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate env_temp_dir() builtin function
/// Returns system temp directory
/// Complexity: 1 (within Toyota Way limits)
fn eval_env_temp_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_temp_dir", args, 0)?;

    let temp = std::env::temp_dir();
    Ok(Value::from_string(temp.to_string_lossy().to_string()))
}

// ==================== FILE SYSTEM FUNCTIONS ====================
// Layer 3 of three-layer builtin pattern (proven from env functions)
// Phase 2: STDLIB_ACCESS_PLAN - File System Module

/// Evaluate fs_read() builtin function
/// Reads file contents and returns as string
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_read(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_read", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::read_to_string(path.as_ref()) {
            Ok(content) => Ok(Value::from_string(content)),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to read file: {}", e))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_read() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate fs_write() builtin function
/// Writes content to file
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_write(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_write", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(content)) => {
            match std::fs::write(path.as_ref(), content.as_ref()) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to write file: {}", e))),
            }
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_write() expects two string arguments".to_string(),
        )),
    }
}

/// Evaluate fs_exists() builtin function
/// Checks if path exists
/// Complexity: 2 (within Toyota Way limits)
fn eval_fs_exists(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_exists", args, 1)?;

    match &args[0] {
        Value::String(path) => Ok(Value::Bool(std::path::Path::new(path.as_ref()).exists())),
        _ => Err(InterpreterError::RuntimeError(
            "fs_exists() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate fs_create_dir() builtin function
/// Creates directory (including parent directories)
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_create_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_create_dir", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::create_dir_all(path.as_ref()) {
            Ok(_) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to create directory: {}", e))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_create_dir() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate fs_remove_file() builtin function
/// Removes a file
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_remove_file(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_remove_file", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::remove_file(path.as_ref()) {
            Ok(_) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to remove file: {}", e))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_remove_file() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate fs_remove_dir() builtin function
/// Removes a directory
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_remove_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_remove_dir", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::remove_dir(path.as_ref()) {
            Ok(_) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to remove directory: {}", e))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_remove_dir() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate fs_copy() builtin function
/// Copies a file from source to destination
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_copy(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_copy", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(from), Value::String(to)) => {
            match std::fs::copy(from.as_ref(), to.as_ref()) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to copy file: {}", e))),
            }
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_copy() expects two string arguments".to_string(),
        )),
    }
}

/// Evaluate fs_rename() builtin function
/// Renames/moves a file
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_rename(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_rename", args, 2)?;

    match (&args[0], &args[1]) {
        (Value::String(from), Value::String(to)) => {
            match std::fs::rename(from.as_ref(), to.as_ref()) {
                Ok(_) => Ok(Value::Nil),
                Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to rename file: {}", e))),
            }
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_rename() expects two string arguments".to_string(),
        )),
    }
}

/// Evaluate fs_metadata() builtin function
/// Returns file metadata as Object
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_metadata(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_metadata", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::metadata(path.as_ref()) {
            Ok(meta) => {
                let mut map = HashMap::new();
                map.insert("size".to_string(), Value::Integer(meta.len() as i64));
                map.insert("is_dir".to_string(), Value::Bool(meta.is_dir()));
                map.insert("is_file".to_string(), Value::Bool(meta.is_file()));
                Ok(Value::Object(Arc::new(map)))
            },
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to get metadata: {}", e))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_metadata() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate fs_read_dir() builtin function
/// Returns directory contents as Array of strings
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_read_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_read_dir", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::read_dir(path.as_ref()) {
            Ok(entries) => {
                let paths: Vec<Value> = entries
                    .filter_map(|e| e.ok())
                    .map(|e| Value::from_string(e.path().display().to_string()))
                    .collect();
                Ok(Value::Array(paths.into()))
            },
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to read directory: {}", e))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_read_dir() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate fs_canonicalize() builtin function
/// Returns absolute path
/// Complexity: 3 (within Toyota Way limits)
fn eval_fs_canonicalize(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_canonicalize", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::fs::canonicalize(path.as_ref()) {
            Ok(canonical) => Ok(Value::from_string(canonical.display().to_string())),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to canonicalize path: {}", e))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "fs_canonicalize() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate fs_is_file() builtin function
/// Checks if path is a file
/// Complexity: 2 (within Toyota Way limits)
fn eval_fs_is_file(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("fs_is_file", args, 1)?;

    match &args[0] {
        Value::String(path) => Ok(Value::Bool(std::path::Path::new(path.as_ref()).is_file())),
        _ => Err(InterpreterError::RuntimeError(
            "fs_is_file() expects a string argument".to_string(),
        )),
    }
}

/// Dispatch file system functions - Part 1
/// Complexity: 5 (cyclomatic 5, cognitive ≤8 - within strict limits)
fn try_eval_fs_part1(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_fs_read__" => Ok(Some(eval_fs_read(args)?)),
        "__builtin_fs_write__" => Ok(Some(eval_fs_write(args)?)),
        "__builtin_fs_exists__" => Ok(Some(eval_fs_exists(args)?)),
        "__builtin_fs_create_dir__" => Ok(Some(eval_fs_create_dir(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch file system functions - Part 2
/// Complexity: 5 (cyclomatic 5, cognitive ≤8 - within strict limits)
fn try_eval_fs_part2(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_fs_remove_file__" => Ok(Some(eval_fs_remove_file(args)?)),
        "__builtin_fs_remove_dir__" => Ok(Some(eval_fs_remove_dir(args)?)),
        "__builtin_fs_copy__" => Ok(Some(eval_fs_copy(args)?)),
        "__builtin_fs_rename__" => Ok(Some(eval_fs_rename(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch file system functions - Part 3
/// Complexity: 5 (cyclomatic 5, cognitive ≤8 - within strict limits)
fn try_eval_fs_part3(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_fs_metadata__" => Ok(Some(eval_fs_metadata(args)?)),
        "__builtin_fs_read_dir__" => Ok(Some(eval_fs_read_dir(args)?)),
        "__builtin_fs_canonicalize__" => Ok(Some(eval_fs_canonicalize(args)?)),
        "__builtin_fs_is_file__" => Ok(Some(eval_fs_is_file(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for file system functions
/// Complexity: 4 (within Toyota Way limits)
fn try_eval_fs_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    if let Some(result) = try_eval_fs_part1(name, args)? {
        return Ok(Some(result));
    }
    if let Some(result) = try_eval_fs_part2(name, args)? {
        return Ok(Some(result));
    }
    try_eval_fs_part3(name, args)
}

// ==================== PATH FUNCTIONS ====================
// Layer 3 of three-layer builtin pattern (proven from env/fs functions)
// Phase 3: STDLIB_ACCESS_PLAN - Path Module (13 functions)

// Helper functions for path operations (reduce cognitive complexity)

/// Helper: path_join operation
/// Complexity: 3 (minimal nesting)
fn eval_path_join(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_join", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(base), Value::String(component)) => {
            let path = std::path::Path::new(base.as_ref()).join(component.as_ref());
            Ok(Value::from_string(path.to_string_lossy().to_string()))
        },
        _ => Err(InterpreterError::RuntimeError("path_join() expects two string arguments".to_string())),
    }
}

/// Helper: Build path from array of string components
/// Complexity: 3 (extracted to reduce nesting)
fn build_path_from_value_components(components: &[Value]) -> Result<std::path::PathBuf, InterpreterError> {
    let mut path = std::path::PathBuf::new();
    for component in components.iter() {
        match component {
            Value::String(s) => path.push(s.as_ref()),
            _ => return Err(InterpreterError::RuntimeError("path_join_many() expects array of strings".to_string())),
        }
    }
    Ok(path)
}

/// Helper: path_join_many operation
/// Complexity: 3 (reduced via helper extraction)
fn eval_path_join_many(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_join_many", args, 1)?;
    match &args[0] {
        Value::Array(components) => {
            let path = build_path_from_value_components(components)?;
            Ok(Value::from_string(path.to_string_lossy().to_string()))
        },
        _ => Err(InterpreterError::RuntimeError("path_join_many() expects an array argument".to_string())),
    }
}

/// Helper: path_parent operation
/// Complexity: 4
fn eval_path_parent(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_parent", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            match p.parent() {
                Some(parent) => Ok(Value::from_string(parent.to_string_lossy().to_string())),
                None => Ok(Value::Nil),
            }
        },
        _ => Err(InterpreterError::RuntimeError("path_parent() expects a string argument".to_string())),
    }
}

/// Helper: path_file_name operation
/// Complexity: 4
fn eval_path_file_name(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_file_name", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            match p.file_name() {
                Some(name) => Ok(Value::from_string(name.to_string_lossy().to_string())),
                None => Ok(Value::Nil),
            }
        },
        _ => Err(InterpreterError::RuntimeError("path_file_name() expects a string argument".to_string())),
    }
}

/// Helper: path_file_stem operation
/// Complexity: 4
fn eval_path_file_stem(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_file_stem", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            match p.file_stem() {
                Some(stem) => Ok(Value::from_string(stem.to_string_lossy().to_string())),
                None => Ok(Value::Nil),
            }
        },
        _ => Err(InterpreterError::RuntimeError("path_file_stem() expects a string argument".to_string())),
    }
}

/// Helper: path_extension operation
/// Complexity: 4
fn eval_path_extension(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_extension", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            match p.extension() {
                Some(ext) => Ok(Value::from_string(ext.to_string_lossy().to_string())),
                None => Ok(Value::Nil),
            }
        },
        _ => Err(InterpreterError::RuntimeError("path_extension() expects a string argument".to_string())),
    }
}

/// Helper: path_is_absolute operation
/// Complexity: 2
fn eval_path_is_absolute(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_is_absolute", args, 1)?;
    match &args[0] {
        Value::String(path) => Ok(Value::Bool(std::path::Path::new(path.as_ref()).is_absolute())),
        _ => Err(InterpreterError::RuntimeError("path_is_absolute() expects a string argument".to_string())),
    }
}

/// Helper: path_is_relative operation
/// Complexity: 2
fn eval_path_is_relative(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_is_relative", args, 1)?;
    match &args[0] {
        Value::String(path) => Ok(Value::Bool(std::path::Path::new(path.as_ref()).is_relative())),
        _ => Err(InterpreterError::RuntimeError("path_is_relative() expects a string argument".to_string())),
    }
}

/// Helper: path_canonicalize operation
/// Complexity: 4
fn eval_path_canonicalize(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_canonicalize", args, 1)?;
    match &args[0] {
        Value::String(path) => match std::fs::canonicalize(path.as_ref()) {
            Ok(canonical) => Ok(Value::from_string(canonical.to_string_lossy().to_string())),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to canonicalize path: {}", e))),
        },
        _ => Err(InterpreterError::RuntimeError("path_canonicalize() expects a string argument".to_string())),
    }
}

/// Helper: path_with_extension operation
/// Complexity: 3
fn eval_path_with_extension(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_with_extension", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(ext)) => {
            let p = std::path::Path::new(path.as_ref()).with_extension(ext.as_ref());
            Ok(Value::from_string(p.to_string_lossy().to_string()))
        },
        _ => Err(InterpreterError::RuntimeError("path_with_extension() expects two string arguments".to_string())),
    }
}

/// Helper: path_with_file_name operation
/// Complexity: 3
fn eval_path_with_file_name(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_with_file_name", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(name)) => {
            let p = std::path::Path::new(path.as_ref()).with_file_name(name.as_ref());
            Ok(Value::from_string(p.to_string_lossy().to_string()))
        },
        _ => Err(InterpreterError::RuntimeError("path_with_file_name() expects two string arguments".to_string())),
    }
}

/// Helper: path_components operation
/// Complexity: 3
fn eval_path_components(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_components", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            let components: Vec<Value> = p.components()
                .map(|c| Value::from_string(c.as_os_str().to_string_lossy().to_string()))
                .collect();
            Ok(Value::Array(components.into()))
        },
        _ => Err(InterpreterError::RuntimeError("path_components() expects a string argument".to_string())),
    }
}

/// Helper: path_normalize operation
/// Complexity: 4
fn eval_path_normalize(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("path_normalize", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            let mut normalized = std::path::PathBuf::new();
            for component in p.components() {
                match component {
                    std::path::Component::CurDir => {},
                    std::path::Component::ParentDir => { normalized.pop(); },
                    _ => normalized.push(component),
                }
            }
            Ok(Value::from_string(normalized.to_string_lossy().to_string()))
        },
        _ => Err(InterpreterError::RuntimeError("path_normalize() expects a string argument".to_string())),
    }
}

/// Dispatch path functions - Part 1 (functions 1-4)
/// Complexity: 5 (reduced via helper extraction)
fn try_eval_path_part1(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_path_join__" => Ok(Some(eval_path_join(args)?)),
        "__builtin_path_join_many__" => Ok(Some(eval_path_join_many(args)?)),
        "__builtin_path_parent__" => Ok(Some(eval_path_parent(args)?)),
        "__builtin_path_file_name__" => Ok(Some(eval_path_file_name(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch path functions - Part 2 (functions 5-8)
/// Complexity: 5 (reduced via helper extraction)
fn try_eval_path_part2(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_path_file_stem__" => Ok(Some(eval_path_file_stem(args)?)),
        "__builtin_path_extension__" => Ok(Some(eval_path_extension(args)?)),
        "__builtin_path_is_absolute__" => Ok(Some(eval_path_is_absolute(args)?)),
        "__builtin_path_is_relative__" => Ok(Some(eval_path_is_relative(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch path functions - Part 3a (functions 9-11)
/// Complexity: 4 (split for cognitive limit)
fn try_eval_path_part3a(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_path_canonicalize__" => Ok(Some(eval_path_canonicalize(args)?)),
        "__builtin_path_with_extension__" => Ok(Some(eval_path_with_extension(args)?)),
        "__builtin_path_with_file_name__" => Ok(Some(eval_path_with_file_name(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch path functions - Part 3b (functions 12-13)
/// Complexity: 3 (split for cognitive limit)
fn try_eval_path_part3b(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_path_components__" => Ok(Some(eval_path_components(args)?)),
        "__builtin_path_normalize__" => Ok(Some(eval_path_normalize(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for path functions
/// Complexity: 4 (loop pattern reduces cognitive load)
fn try_eval_path_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    let dispatchers: &[fn(&str, &[Value]) -> Result<Option<Value>, InterpreterError>] = &[
        try_eval_path_part1,
        try_eval_path_part2,
        try_eval_path_part3a,
        try_eval_path_part3b,
    ];

    for dispatcher in dispatchers {
        if let Some(result) = dispatcher(name, args)? {
            return Ok(Some(result));
        }
    }
    Ok(None)
}

// ==================== JSON FUNCTIONS ====================
// Layer 3 of three-layer builtin pattern (proven from env/fs/path functions)
// Phase 4: STDLIB_ACCESS_PLAN - JSON Module (10 functions)

// Helper functions for JSON operations (reduce cognitive complexity)

/// Helper: json_parse operation
/// Complexity: 3
fn eval_json_parse(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_parse", args, 1)?;
    match &args[0] {
        Value::String(s) => {
            match serde_json::from_str::<serde_json::Value>(s) {
                Ok(json) => {
                    // Convert serde_json::Value to Ruchy Value
                    fn json_to_value(json: serde_json::Value) -> Value {
                        match json {
                            serde_json::Value::Null => Value::Nil,
                            serde_json::Value::Bool(b) => Value::Bool(b),
                            serde_json::Value::Number(n) => {
                                if let Some(i) = n.as_i64() {
                                    Value::Integer(i)
                                } else if let Some(f) = n.as_f64() {
                                    Value::Float(f)
                                } else {
                                    Value::Nil
                                }
                            },
                            serde_json::Value::String(s) => Value::from_string(s),
                            serde_json::Value::Array(arr) => {
                                let values: Vec<Value> = arr.into_iter().map(json_to_value).collect();
                                Value::Array(values.into())
                            },
                            serde_json::Value::Object(obj) => {
                                let mut map = std::collections::HashMap::new();
                                for (k, v) in obj {
                                    map.insert(k, json_to_value(v));
                                }
                                Value::Object(std::sync::Arc::new(map))
                            },
                        }
                    }
                    Ok(json_to_value(json))
                },
                Err(e) => Err(InterpreterError::RuntimeError(format!("JSON parse error: {}", e))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("json_parse() expects a string argument".to_string())),
    }
}

/// Helper: Convert Ruchy Value to serde_json::Value
/// Complexity: 3
fn value_to_json(value: &Value) -> Result<serde_json::Value, InterpreterError> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::json!(*i)),
        Value::Float(f) => Ok(serde_json::json!(*f)),
        Value::String(s) => Ok(serde_json::Value::String(s.to_string())),
        Value::Array(arr) => {
            let json_arr: Result<Vec<serde_json::Value>, _> = arr.iter()
                .map(value_to_json)
                .collect();
            Ok(serde_json::Value::Array(json_arr?))
        },
        Value::Object(map) => {
            let mut json_obj = serde_json::Map::new();
            for (k, v) in map.iter() {
                json_obj.insert(k.clone(), value_to_json(v)?);
            }
            Ok(serde_json::Value::Object(json_obj))
        },
        _ => Err(InterpreterError::RuntimeError(format!("Cannot convert {:?} to JSON", value))),
    }
}

/// Helper: json_stringify operation
/// Complexity: 2
fn eval_json_stringify(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_stringify", args, 1)?;
    let json = value_to_json(&args[0])?;
    match serde_json::to_string(&json) {
        Ok(s) => Ok(Value::from_string(s)),
        Err(e) => Err(InterpreterError::RuntimeError(format!("JSON stringify error: {}", e))),
    }
}

/// Helper: json_pretty operation
/// Complexity: 2
fn eval_json_pretty(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_pretty", args, 1)?;
    let json = value_to_json(&args[0])?;
    match serde_json::to_string_pretty(&json) {
        Ok(s) => Ok(Value::from_string(s)),
        Err(e) => Err(InterpreterError::RuntimeError(format!("JSON pretty error: {}", e))),
    }
}

/// Helper: json_read operation
/// Complexity: 3
fn eval_json_read(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_read", args, 1)?;
    match &args[0] {
        Value::String(path) => {
            let content = std::fs::read_to_string(path.as_ref())
                .map_err(|e| InterpreterError::RuntimeError(format!("Failed to read file: {}", e)))?;
            eval_json_parse(&[Value::from_string(content)])
        },
        _ => Err(InterpreterError::RuntimeError("json_read() expects a string argument".to_string())),
    }
}

/// Helper: json_write operation
/// Complexity: 3
fn eval_json_write(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_write", args, 2)?;
    match &args[0] {
        Value::String(path) => {
            let json = value_to_json(&args[1])?;
            let content = serde_json::to_string_pretty(&json)
                .map_err(|e| InterpreterError::RuntimeError(format!("JSON stringify error: {}", e)))?;
            std::fs::write(path.as_ref(), content)
                .map_err(|e| InterpreterError::RuntimeError(format!("Failed to write file: {}", e)))?;
            Ok(Value::Bool(true))
        },
        _ => Err(InterpreterError::RuntimeError("json_write() expects first argument to be string".to_string())),
    }
}

/// Helper: json_validate operation
/// Complexity: 2
fn eval_json_validate(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_validate", args, 1)?;
    match &args[0] {
        Value::String(s) => {
            let is_valid = serde_json::from_str::<serde_json::Value>(s).is_ok();
            Ok(Value::Bool(is_valid))
        },
        _ => Err(InterpreterError::RuntimeError("json_validate() expects a string argument".to_string())),
    }
}

/// Helper: json_type operation
/// Complexity: 3
fn eval_json_type(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_type", args, 1)?;
    match &args[0] {
        Value::String(s) => {
            match serde_json::from_str::<serde_json::Value>(s) {
                Ok(json) => {
                    let type_str = match json {
                        serde_json::Value::Null => "null",
                        serde_json::Value::Bool(_) => "boolean",
                        serde_json::Value::Number(_) => "number",
                        serde_json::Value::String(_) => "string",
                        serde_json::Value::Array(_) => "array",
                        serde_json::Value::Object(_) => "object",
                    };
                    Ok(Value::from_string(type_str.to_string()))
                },
                Err(e) => Err(InterpreterError::RuntimeError(format!("JSON parse error: {}", e))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("json_type() expects a string argument".to_string())),
    }
}

/// Helper: json_merge operation
/// Complexity: 2
fn eval_json_merge(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_merge", args, 2)?;
    let json1 = value_to_json(&args[0])?;
    let json2 = value_to_json(&args[1])?;

    fn merge_json(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
        match (a, b) {
            (serde_json::Value::Object(mut a_map), serde_json::Value::Object(b_map)) => {
                for (k, v) in b_map {
                    if let Some(a_val) = a_map.get_mut(&k) {
                        *a_val = merge_json(a_val.clone(), v);
                    } else {
                        a_map.insert(k, v);
                    }
                }
                serde_json::Value::Object(a_map)
            },
            (_, b_val) => b_val,
        }
    }

    let merged = merge_json(json1, json2);
    eval_json_parse(&[Value::from_string(merged.to_string())])
}

/// Helper: json_get operation
/// Complexity: 3
fn eval_json_get(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_get", args, 2)?;
    let json = value_to_json(&args[0])?;

    match &args[1] {
        Value::String(path) => {
            let parts: Vec<&str> = path.split('.').collect();

            fn get_path<'a>(json: &'a serde_json::Value, path: &[&str]) -> Option<&'a serde_json::Value> {
                if path.is_empty() {
                    return Some(json);
                }
                match json {
                    serde_json::Value::Object(map) => {
                        map.get(path[0]).and_then(|v| get_path(v, &path[1..]))
                    },
                    _ => None,
                }
            }

            match get_path(&json, &parts) {
                Some(val) => eval_json_parse(&[Value::from_string(val.to_string())]),
                None => Ok(Value::Nil),
            }
        },
        _ => Err(InterpreterError::RuntimeError("json_get() expects second argument to be string".to_string())),
    }
}

/// Helper: json_set operation
/// Complexity: 3
fn eval_json_set(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("json_set", args, 3)?;
    let mut json = value_to_json(&args[0])?;
    let new_value = value_to_json(&args[2])?;

    match &args[1] {
        Value::String(path) => {
            let parts: Vec<&str> = path.split('.').collect();

            fn set_path(json: &mut serde_json::Value, path: &[&str], value: serde_json::Value) {
                if path.is_empty() {
                    *json = value;
                    return;
                }

                if path.len() == 1 {
                    if let serde_json::Value::Object(map) = json {
                        map.insert(path[0].to_string(), value);
                    }
                } else if let serde_json::Value::Object(map) = json {
                    if let Some(next) = map.get_mut(path[0]) {
                        set_path(next, &path[1..], value);
                    }
                }
            }

            set_path(&mut json, &parts, new_value);
            eval_json_parse(&[Value::from_string(json.to_string())])
        },
        _ => Err(InterpreterError::RuntimeError("json_set() expects second argument to be string".to_string())),
    }
}

/// Dispatch JSON functions - Part 1 (functions 1-5)
/// Complexity: 5
fn try_eval_json_part1(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_json_parse__" => Ok(Some(eval_json_parse(args)?)),
        "__builtin_json_stringify__" => Ok(Some(eval_json_stringify(args)?)),
        "__builtin_json_pretty__" => Ok(Some(eval_json_pretty(args)?)),
        "__builtin_json_read__" => Ok(Some(eval_json_read(args)?)),
        "__builtin_json_write__" => Ok(Some(eval_json_write(args)?)),
        _ => Ok(None),
    }
}

/// Dispatch JSON functions - Part 2 (functions 6-10)
/// Complexity: 6
fn try_eval_json_part2(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_json_validate__" => Ok(Some(eval_json_validate(args)?)),
        "__builtin_json_type__" => Ok(Some(eval_json_type(args)?)),
        "__builtin_json_merge__" => Ok(Some(eval_json_merge(args)?)),
        "__builtin_json_get__" => Ok(Some(eval_json_get(args)?)),
        "__builtin_json_set__" => Ok(Some(eval_json_set(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for JSON functions
/// Complexity: 4
fn try_eval_json_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    let dispatchers: &[fn(&str, &[Value]) -> Result<Option<Value>, InterpreterError>] = &[
        try_eval_json_part1,
        try_eval_json_part2,
    ];

    for dispatcher in dispatchers {
        if let Some(result) = dispatcher(name, args)? {
            return Ok(Some(result));
        }
    }
    Ok(None)
}

// ==============================================================================
// HTTP Builtin Functions (STDLIB-PHASE-5)
// ==============================================================================

/// Dispatcher for HTTP builtin functions
/// Complexity: 2 (loop + match delegation)
fn try_eval_http_function(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "http_get" => Ok(Some(eval_http_get(args)?)),
        "http_post" => Ok(Some(eval_http_post(args)?)),
        "http_put" => Ok(Some(eval_http_put(args)?)),
        "http_delete" => Ok(Some(eval_http_delete(args)?)),
        _ => Ok(None),
    }
}

/// Eval: http_get(url)
/// Complexity: 2 (validation + stdlib delegation)
fn eval_http_get(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("http_get", args, 1)?;
    match &args[0] {
        Value::String(url) => {
            match crate::stdlib::http::get(url) {
                Ok(response) => Ok(Value::from_string(response)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("HTTP GET failed: {}", e))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("http_get() expects a string URL".to_string())),
    }
}

/// Eval: http_post(url, body)
/// Complexity: 2 (validation + stdlib delegation)
fn eval_http_post(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("http_post", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(url), Value::String(body)) => {
            match crate::stdlib::http::post(url, body) {
                Ok(response) => Ok(Value::from_string(response)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("HTTP POST failed: {}", e))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("http_post() expects two string arguments".to_string())),
    }
}

/// Eval: http_put(url, body)
/// Complexity: 2 (validation + stdlib delegation)
fn eval_http_put(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("http_put", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(url), Value::String(body)) => {
            match crate::stdlib::http::put(url, body) {
                Ok(response) => Ok(Value::from_string(response)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("HTTP PUT failed: {}", e))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("http_put() expects two string arguments".to_string())),
    }
}

/// Eval: http_delete(url)
/// Complexity: 2 (validation + stdlib delegation)
fn eval_http_delete(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("http_delete", args, 1)?;
    match &args[0] {
        Value::String(url) => {
            match crate::stdlib::http::delete(url) {
                Ok(response) => Ok(Value::from_string(response)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("HTTP DELETE failed: {}", e))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("http_delete() expects a string URL".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_sqrt() {
        let args = vec![Value::Integer(16)];
        let result = eval_sqrt(&args).unwrap();
        assert_eq!(result, Value::Float(4.0));

        let args = vec![Value::Float(9.0)];
        let result = eval_sqrt(&args).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_eval_pow() {
        let args = vec![Value::Integer(2), Value::Integer(3)];
        let result = eval_pow(&args).unwrap();
        assert_eq!(result, Value::Integer(8));

        let args = vec![Value::Float(2.0), Value::Float(3.0)];
        let result = eval_pow(&args).unwrap();
        assert_eq!(result, Value::Float(8.0));
    }

    #[test]
    fn test_eval_abs() {
        let args = vec![Value::Integer(-42)];
        let result = eval_abs(&args).unwrap();
        assert_eq!(result, Value::Integer(42));

        let args = vec![Value::Float(-3.14)];
        let result = eval_abs(&args).unwrap();
        assert_eq!(result, Value::Float(3.14));
    }

    #[test]
    fn test_eval_min_max() {
        let args = vec![Value::Integer(5), Value::Integer(3)];
        let min_result = eval_min(&args).unwrap();
        assert_eq!(min_result, Value::Integer(3));

        let max_result = eval_max(&args).unwrap();
        assert_eq!(max_result, Value::Integer(5));
    }

    #[test]
    fn test_eval_len() {
        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_len(&args).unwrap();
        assert_eq!(result, Value::Integer(5));

        let args = vec![Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))];
        let result = eval_len(&args).unwrap();
        assert_eq!(result, Value::Integer(3));
    }

    #[test]
    fn test_eval_type() {
        let args = vec![Value::Integer(42)];
        let result = eval_type(&args).unwrap();
        assert_eq!(result, Value::from_string("integer".to_string()));

        let args = vec![Value::Float(3.14)];
        let result = eval_type(&args).unwrap();
        assert_eq!(result, Value::from_string("float".to_string()));
    }

    #[test]
    fn test_eval_range() {
        let args = vec![Value::Integer(3)];
        let result = eval_range(&args).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(0));
            assert_eq!(arr[1], Value::Integer(1));
            assert_eq!(arr[2], Value::Integer(2));
        } else {
            panic!("Expected array result");
        }
    }

    #[test]
    fn test_eval_reverse() {
        let args = vec![Value::Array(Arc::from(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]))];
        let result = eval_reverse(&args).unwrap();
        if let Value::Array(arr) = result {
            assert_eq!(arr[0], Value::Integer(3));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(1));
        } else {
            panic!("Expected array result");
        }

        let args = vec![Value::from_string("hello".to_string())];
        let result = eval_reverse(&args).unwrap();
        assert_eq!(result, Value::from_string("olleh".to_string()));
    }
}
