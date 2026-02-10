//! Builtin function evaluation module
//!
//! This module handles all builtin functions including math operations,
//! I/O functions, utility functions, and type operations.
//! Extracted for maintainability and following Toyota Way principles.
//! All functions maintain <10 cyclomatic complexity.
//!
//! Sub-modules:
//! - `eval_builtin_fs`: Filesystem operations (read, write, walk, glob, search)
//! - `eval_builtin_path`: Path manipulation (join, parent, extension, normalize)
//! - `eval_builtin_json_ops`: JSON operations (parse, stringify, merge, get, set)
//! - `eval_builtin_platform`: HTTP, HTML, Process, File, String, and type conversions

use crate::runtime::validation::validate_arg_count;
use crate::runtime::{InterpreterError, Value};

use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;


// Re-export from sub-modules so that `use super::*` in test modules still works
pub(crate) use crate::runtime::eval_builtin_fs::*;
pub(crate) use crate::runtime::eval_builtin_json_ops::*;
pub(crate) use crate::runtime::eval_builtin_path::*;
pub(crate) use crate::runtime::eval_builtin_platform::*;

pub fn eval_builtin_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    type Handler = fn(&str, &[Value]) -> Result<Option<Value>, InterpreterError>;

    // Platform-independent handlers dispatched via table lookup
    let handlers: &[Handler] = &[
        try_eval_io_function,
        try_eval_math_function,
        try_eval_utility_function,
        try_eval_collection_function,
        try_eval_conversion_function,
        try_eval_time_function,
        try_eval_dataframe_function,
        try_eval_environment_function,
        try_eval_fs_function,
        try_eval_stdlib003,
        try_eval_stdlib005,
        try_eval_path_function,
        try_eval_json_function,
        try_eval_file_function,
        try_eval_string_function,
    ];

    for handler in handlers {
        if let Some(result) = handler(name, args)? {
            return Ok(Some(result));
        }
    }

    // Platform-specific handlers (not available in WASM)
    #[cfg(not(target_arch = "wasm32"))]
    {
        let platform_handlers: &[Handler] = &[
            try_eval_http_function,
            try_eval_html_function,
            try_eval_process_function,
        ];
        for handler in platform_handlers {
            if let Some(result) = handler(name, args)? {
                return Ok(Some(result));
            }
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
fn try_eval_basic_math_part1(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_sqrt__" => Ok(Some(eval_sqrt(args)?)),
        "__builtin_pow__" => Ok(Some(eval_pow(args)?)),
        "__builtin_abs__" => Ok(Some(eval_abs(args)?)),
        _ => Ok(None),
    }
}

/// Basic math functions - Part 2
/// Complexity: 3 (within Toyota Way limits)
fn try_eval_basic_math_part2(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
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
fn try_eval_advanced_math_part1(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_floor__" => Ok(Some(eval_floor(args)?)),
        "__builtin_ceil__" => Ok(Some(eval_ceil(args)?)),
        "__builtin_round__" => Ok(Some(eval_round(args)?)),
        _ => Ok(None),
    }
}

/// Advanced math functions - Part 2 (trigonometry)
/// Complexity: 4 (within Toyota Way limits)
fn try_eval_advanced_math_part2(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_sin__" => Ok(Some(eval_sin(args)?)),
        "__builtin_cos__" => Ok(Some(eval_cos(args)?)),
        "__builtin_tan__" => Ok(Some(eval_tan(args)?)),
        _ => Ok(None),
    }
}

/// Advanced math functions - Part 3 (logarithms, random, exp) - STDLIB-002, QA-065
/// Complexity: 5 (within Toyota Way limits)
fn try_eval_advanced_math_part3(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_log__" => Ok(Some(eval_log(args)?)),
        "__builtin_log10__" => Ok(Some(eval_log10(args)?)),
        "__builtin_exp__" => Ok(Some(eval_exp(args)?)),
        "__builtin_random__" => Ok(Some(eval_random(args)?)),
        _ => Ok(None),
    }
}

/// Dispatcher for advanced math functions
/// Complexity: 4 (within Toyota Way limits) - STDLIB-002: Added part3
fn try_eval_advanced_math(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    if let Some(result) = try_eval_advanced_math_part1(name, args)? {
        return Ok(Some(result));
    }
    if let Some(result) = try_eval_advanced_math_part2(name, args)? {
        return Ok(Some(result));
    }
    try_eval_advanced_math_part3(name, args)
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
/// Complexity: 9 (within Toyota Way limits, added type inspection functions)
fn try_eval_utility_part2(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_type__" => Ok(Some(eval_type(args)?)),
        "__builtin_type_of__" => Ok(Some(eval_type_of(args)?)),
        "__builtin_is_nil__" => Ok(Some(eval_is_nil(args)?)),
        "__builtin_reverse__" => Ok(Some(eval_reverse(args)?)),
        // Test assertion built-ins for unit testing support
        "__builtin_assert_eq__" => Ok(Some(eval_assert_eq(args)?)),
        "__builtin_assert__" => Ok(Some(eval_assert(args)?)),
        // Advanced array utilities for functional programming patterns
        "__builtin_zip__" => Ok(Some(eval_zip(args)?)),
        "__builtin_enumerate__" => Ok(Some(eval_enumerate(args)?)),
        _ => Ok(None),
    }
}

/// Collection mutation functions (push, pop, sort)
/// Complexity: 4 (within Toyota Way limits)
fn try_eval_collection_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_push__" => Ok(Some(eval_push(args)?)),
        "__builtin_pop__" => Ok(Some(eval_pop(args)?)),
        "__builtin_sort__" => Ok(Some(eval_sort(args)?)),
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

/// Try to evaluate type conversion functions (STDLIB-001)
///
/// Wraps Rust stdlib methods for zero-cost abstraction.
/// Complexity: 8 (within Toyota Way limits, added `to_string`)
fn try_eval_conversion_function(
    name: &str,
    args: &[Value],
) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_str__" => Ok(Some(eval_str(args)?)),
        "__builtin_to_string__" => Ok(Some(eval_to_string(args)?)),
        "__builtin_int__" => Ok(Some(eval_int(args)?)),
        "__builtin_float__" => Ok(Some(eval_float(args)?)),
        "__builtin_bool__" => Ok(Some(eval_bool(args)?)),
        "__builtin_parse_int__" => Ok(Some(eval_parse_int(args)?)),
        "__builtin_parse_float__" => Ok(Some(eval_parse_float(args)?)),
        _ => Ok(None),
    }
}

fn try_eval_time_function(name: &str, args: &[Value]) -> Result<Option<Value>, InterpreterError> {
    match name {
        "__builtin_sleep__" => Ok(Some(eval_sleep(args)?)),
        "__builtin_timestamp__" => Ok(Some(eval_timestamp(args)?)),
        "__builtin_chrono_utc_now__" => Ok(Some(eval_chrono_utc_now(args)?)),
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
/// Format value for println (strings without quotes)
/// Complexity: 2 (within Toyota Way limits)
fn format_value_for_println(value: &Value) -> String {
    match value {
        Value::String(s) => s.to_string(),
        other => format!("{other}"),
    }
}

/// Format string with interpolation
/// Complexity: 2 (within Toyota Way limits)
fn format_with_interpolation(fmt_str: &str, args: &[Value]) -> String {
    let mut result = fmt_str.to_string();
    for arg in args {
        if let Some(pos) = result.find("{}") {
            result.replace_range(pos..pos + 2, &format_value_for_println(arg));
        }
    }
    result
}

/// Join values with spaces
/// Complexity: 1 (within Toyota Way limits)
fn join_values(args: &[Value]) -> String {
    args.iter()
        .map(format_value_for_println)
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
    let _ = std::io::stdout().flush();

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
    let _ = std::io::stdout().flush();

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

// ============================================================================
// STDLIB-002: Advanced Math Functions - Logarithms and Random
// Zero-cost abstraction wrapping Rust std::f64 methods
// ============================================================================

/// Natural logarithm (base e)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_log(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("log", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).ln())), // Wraps Rust f64::ln
        Value::Float(f) => Ok(Value::Float(f.ln())),
        _ => Err(InterpreterError::RuntimeError(
            "log() expects a number".to_string(),
        )),
    }
}

/// Base-10 logarithm
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_log10(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("log10", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).log10())), // Wraps Rust f64::log10
        Value::Float(f) => Ok(Value::Float(f.log10())),
        _ => Err(InterpreterError::RuntimeError(
            "log10() expects a number".to_string(),
        )),
    }
}

/// Exponential function (e^x) - QA-065
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_exp(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("exp", args, 1)?;
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).exp())), // Wraps Rust f64::exp
        Value::Float(f) => Ok(Value::Float(f.exp())),
        _ => Err(InterpreterError::RuntimeError(
            "exp() expects a number".to_string(),
        )),
    }
}

/// Generate random float in [0.0, 1.0)
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn eval_random(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("random", args, 0)?;
    // Wraps Rust rand::random (zero-cost abstraction)
    use rand::Rng;
    let mut rng = rand::thread_rng();
    Ok(Value::Float(rng.gen::<f64>())) // Returns [0.0, 1.0)
}

/// Length of collections and strings
///
/// # Complexity
/// Cyclomatic complexity: 6 (within Toyota Way limits)
fn eval_len(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("len", args, 1)?;
    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        Value::Tuple(t) => Ok(Value::Integer(t.len() as i64)),
        Value::DataFrame { columns } => {
            if columns.is_empty() {
                Ok(Value::Integer(0))
            } else {
                Ok(Value::Integer(columns[0].values.len() as i64))
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "len() expects a string, array, tuple, or dataframe".to_string(),
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

/// Get type name of a value (alias for `eval_type`)
/// RUNTIME-BUG-001: Added to support `type_of()` function
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_type_of(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("type_of", args, 1)?;
    Ok(Value::from_string(args[0].type_name().to_string()))
}

/// Check if value is nil
/// RUNTIME-BUG-001: Added to support `is_nil()` function
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_is_nil(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("is_nil", args, 1)?;
    Ok(Value::Bool(matches!(args[0], Value::Nil)))
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

/// Push element to array (returns new array)
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
fn eval_push(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("push", args, 2)?;
    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.to_vec();
            new_arr.push(args[1].clone());
            Ok(Value::from_array(new_arr))
        }
        _ => Err(InterpreterError::RuntimeError(
            "push() expects an array as first argument".to_string(),
        )),
    }
}

/// Pop element from array (returns element, not mutated array)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within Toyota Way limits)
fn eval_pop(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("pop", args, 1)?;
    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.to_vec();
            if let Some(val) = new_arr.pop() {
                Ok(val)
            } else {
                Ok(Value::nil())
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "pop() expects an array".to_string(),
        )),
    }
}

/// Sort array (returns new sorted array)
///
/// # Complexity
/// Cyclomatic complexity: 4 (within Toyota Way limits)
fn eval_sort(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("sort", args, 1)?;
    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.to_vec();
            new_arr.sort_by(|a, b| match (a, b) {
                (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                (Value::Float(x), Value::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => x.cmp(y),
                _ => std::cmp::Ordering::Equal,
            });
            Ok(Value::from_array(new_arr))
        }
        _ => Err(InterpreterError::RuntimeError(
            "sort() expects an array".to_string(),
        )),
    }
}

// ============================================================================
// STDLIB-004: Advanced Array Utility Functions
// ============================================================================

/// Zip two arrays into array of tuples
/// Complexity: 3 (within Toyota Way limits)
fn eval_zip(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("zip", args, 2)?;
    match (&args[0], &args[1]) {
        (Value::Array(a), Value::Array(b)) => {
            let zipped: Vec<Value> = a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| Value::Tuple(Arc::from(vec![x.clone(), y.clone()].as_slice())))
                .collect();
            Ok(Value::from_array(zipped))
        }
        _ => Err(InterpreterError::RuntimeError(
            "zip() expects two arrays".to_string(),
        )),
    }
}

/// Enumerate array (add indices)
/// Complexity: 2 (within Toyota Way limits)
fn eval_enumerate(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("enumerate", args, 1)?;
    match &args[0] {
        Value::Array(arr) => {
            let enumerated: Vec<Value> = arr
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    Value::Tuple(Arc::from(
                        vec![Value::Integer(i as i64), v.clone()].as_slice(),
                    ))
                })
                .collect();
            Ok(Value::from_array(enumerated))
        }
        _ => Err(InterpreterError::RuntimeError(
            "enumerate() expects an array".to_string(),
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

/// `chrono::Utc::now()` - Get current UTC time
///
/// Returns a string representation of the current UTC timestamp in RFC3339 format.
/// This implements the `chrono::Utc::now()` functionality for Issue #82.
///
/// # Complexity
/// Cyclomatic complexity: 2 (within Toyota Way limits)
///
/// # Examples
/// ```ignore
/// let now = Utc::now();
/// println!("Current UTC time: {}", now);
/// ```
fn eval_chrono_utc_now(args: &[Value]) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "Utc::now() expects no arguments".to_string(),
        ));
    }

    // Get current UTC time using chrono
    let now = chrono::Utc::now();
    let timestamp_str = now.to_rfc3339();

    Ok(Value::from_string(timestamp_str))
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

/// Evaluate `env_args()` builtin function
/// Returns command-line arguments as an array of strings
/// Complexity: 2 (within Toyota Way limits)
fn eval_env_args(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_args", args, 0)?;

    // Get command-line arguments
    let cmd_args: Vec<Value> = std::env::args().map(Value::from_string).collect();

    Ok(Value::from_array(cmd_args))
}

/// Evaluate `env_var()` builtin function
/// Returns Result enum: Ok(value) or Err(NotFound)
/// Issue #96: Match Rust `std::env::var()` API (returns Result)
/// Complexity: 5 (within Toyota Way limits)
fn eval_env_var(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_var", args, 1)?;

    match &args[0] {
        Value::String(key) => {
            // Return Result enum to match Rust API
            match std::env::var(key.as_ref()) {
                Ok(val) => {
                    // Ok(value)
                    Ok(Value::EnumVariant {
                        enum_name: "Result".to_string(),
                        variant_name: "Ok".to_string(),
                        data: Some(vec![Value::from_string(val)]),
                    })
                }
                Err(_) => {
                    // Err(NotFound)
                    Ok(Value::EnumVariant {
                        enum_name: "Result".to_string(),
                        variant_name: "Err".to_string(),
                        data: Some(vec![Value::from_string("NotFound".to_string())]),
                    })
                }
            }
        }
        _ => Err(InterpreterError::RuntimeError(
            "env_var() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `env_set_var()` builtin function
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

/// Evaluate `env_remove_var()` builtin function
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

/// Evaluate `env_vars()` builtin function
/// Returns all environment variables as `HashMap`
/// Complexity: 1 (within Toyota Way limits)
fn eval_env_vars(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_vars", args, 0)?;

    let vars: HashMap<String, Value> = std::env::vars()
        .map(|(k, v)| (k, Value::from_string(v)))
        .collect();

    Ok(Value::Object(Arc::new(vars)))
}

/// Evaluate `env_current_dir()` builtin function
/// Returns current working directory
/// Complexity: 2 (within Toyota Way limits)
fn eval_env_current_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_current_dir", args, 0)?;

    match std::env::current_dir() {
        Ok(path) => Ok(Value::from_string(path.to_string_lossy().to_string())),
        Err(e) => Err(InterpreterError::RuntimeError(format!(
            "Failed to get current directory: {e}"
        ))),
    }
}

/// Evaluate `env_set_current_dir()` builtin function
/// Changes current working directory
/// Complexity: 2 (within Toyota Way limits)
fn eval_env_set_current_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    validate_arg_count("env_set_current_dir", args, 1)?;

    match &args[0] {
        Value::String(path) => match std::env::set_current_dir(path.as_ref()) {
            Ok(()) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!(
                "Failed to set current directory: {e}"
            ))),
        },
        _ => Err(InterpreterError::RuntimeError(
            "env_set_current_dir() expects a string argument".to_string(),
        )),
    }
}

/// Evaluate `env_temp_dir()` builtin function
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

/// Evaluate `fs_read()` builtin function
/// Reads file contents and returns as string
/// Complexity: 3 (within Toyota Way limits)


#[cfg(test)]
#[path = "eval_builtin_unit_tests.rs"]
mod tests;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "eval_builtin_prop_tests.rs"]
mod property_tests_builtin;