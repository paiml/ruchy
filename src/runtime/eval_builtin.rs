//! Builtin function evaluation module
//!
//! This module handles all builtin functions including math operations,
//! I/O functions, utility functions, and type operations.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.

use crate::runtime::validation::validate_arg_count;
use crate::runtime::{InterpreterError, Value};

#[cfg(test)]
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
fn try_eval_basic_math(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_sqrt__" => Ok(Some(eval_sqrt(args)?)),
        "__builtin_pow__" => Ok(Some(eval_pow(args)?)),
        "__builtin_abs__" => Ok(Some(eval_abs(args)?)),
        "__builtin_min__" => Ok(Some(eval_min(args)?)),
        "__builtin_max__" => Ok(Some(eval_max(args)?)),
        _ => Ok(None),
    }
}

/// Advanced math functions (floor, ceil, round, trig)
fn try_eval_advanced_math(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_floor__" => Ok(Some(eval_floor(args)?)),
        "__builtin_ceil__" => Ok(Some(eval_ceil(args)?)),
        "__builtin_round__" => Ok(Some(eval_round(args)?)),
        "__builtin_sin__" => Ok(Some(eval_sin(args)?)),
        "__builtin_cos__" => Ok(Some(eval_cos(args)?)),
        "__builtin_tan__" => Ok(Some(eval_tan(args)?)),
        _ => Ok(None),
    }
}

fn try_eval_utility_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_len__" => Ok(Some(eval_len(args)?)),
        "__builtin_range__" => Ok(Some(eval_range(args)?)),
        "__builtin_type__" => Ok(Some(eval_type(args)?)),
        "__builtin_reverse__" => Ok(Some(eval_reverse(args)?)),
        _ => Ok(None),
    }
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
fn eval_println(args: &[Value]) -> Result<Value, InterpreterError> {
    let output = if args.is_empty() {
        "\n".to_string()
    } else if let Value::String(fmt_str) = &args[0] {
        if fmt_str.contains("{}") {
            // Perform string interpolation
            let mut result = fmt_str.to_string();
            for arg in &args[1..] {
                if let Some(pos) = result.find("{}") {
                    result.replace_range(pos..pos + 2, &format!("{arg}"));
                }
            }
            format!("{result}\n")
        } else {
            // Fallback: join all args with spaces (backward compatibility)
            let text = args
                .iter()
                .map(|v| format!("{v}"))
                .collect::<Vec<_>>()
                .join(" ");
            format!("{text}\n")
        }
    } else {
        // Fallback: join all args with spaces (backward compatibility)
        let text = args
            .iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!("{text}\n")
    };

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
            let mut result = Vec::new();
            if *step > 0 {
                let mut i = *start;
                while i < *end {
                    result.push(Value::Integer(i));
                    i += step;
                }
            } else {
                let mut i = *start;
                while i > *end {
                    result.push(Value::Integer(i));
                    i += step;
                }
            }
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
fn process_json_char(ch: char, state: &mut JsonParserState, objects: &mut Vec<String>) {
    match ch {
        '"' => state.in_string = !state.in_string,
        '{' if !state.in_string => {
            state.brace_count += 1;
            state.current.push(ch);
        }
        '}' if !state.in_string => {
            state.brace_count -= 1;
            state.current.push(ch);
            if state.brace_count == 0 {
                objects.push(state.current.trim().to_string());
                state.current.clear();
            }
        }
        ',' if !state.in_string && state.brace_count == 0 => {}
        _ => {
            if state.brace_count > 0 {
                state.current.push(ch);
            }
        }
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
fn parse_json_object(obj_str: &str) -> Result<Vec<(String, Value)>, InterpreterError> {
    let mut pairs = Vec::new();
    let inner = obj_str.trim().trim_start_matches('{').trim_end_matches('}');

    for pair in inner.split(',') {
        if let Some(colon_pos) = pair.find(':') {
            let key = pair[..colon_pos].trim().trim_matches('"').to_string();
            let value_str = pair[colon_pos + 1..].trim();

            let value = if value_str.starts_with('"') {
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
            };

            pairs.push((key, value));
        }
    }

    Ok(pairs)
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
