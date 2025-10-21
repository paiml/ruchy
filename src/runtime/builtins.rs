//! Built-in functions module
//!
//! This module contains all built-in functions like println, len, `type_of`, etc.
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

// Global output buffer for capturing println/print output
// Uses Mutex for thread-safety across tokio::spawn_blocking boundaries
pub static OUTPUT_BUFFER: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

/// Enable output capture mode (for notebook/testing)
pub fn enable_output_capture() {
    if let Ok(mut buf) = OUTPUT_BUFFER.lock() {
        buf.clear();
    }
}

/// Get and clear captured output
pub fn get_captured_output() -> String {
    if let Ok(mut buf) = OUTPUT_BUFFER.lock() {
        let output = buf.clone();
        buf.clear();
        output
    } else {
        String::new()
    }
}

/// Check if output capture is enabled
pub fn is_output_capture_enabled() -> bool {
    // For now, always capture if buffer exists
    true
}

/// Registry of built-in functions
#[derive(Debug)]
pub struct BuiltinRegistry {
    functions: HashMap<String, BuiltinFunction>,
}

/// A built-in function signature
pub type BuiltinFunction = fn(&[Value]) -> Result<Value, InterpreterError>;

impl Default for BuiltinRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BuiltinRegistry {
    /// Create a new builtin registry with all standard functions
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
        };
        registry.register_all();
        registry
    }

    /// Register all built-in functions
    ///
    /// # Complexity
    /// Cyclomatic complexity: 1 (within limit of 10)
    fn register_all(&mut self) {
        // I/O functions
        self.register("println", builtin_println);
        self.register("print", builtin_print);
        self.register("dbg", builtin_dbg);

        // Type and inspection functions
        self.register("len", builtin_len);
        self.register("type_of", builtin_type_of);
        self.register("is_nil", builtin_is_nil);

        // Testing/assertion functions
        self.register("assert_eq", builtin_assert_eq);
        self.register("assert", builtin_assert);

        // Math functions
        self.register("sqrt", builtin_sqrt);
        self.register("pow", builtin_pow);
        self.register("abs", builtin_abs);
        self.register("min", builtin_min);
        self.register("max", builtin_max);
        self.register("floor", builtin_floor);
        self.register("ceil", builtin_ceil);
        self.register("round", builtin_round);

        // String functions
        self.register("to_string", builtin_to_string);
        self.register("parse_int", builtin_parse_int);
        self.register("parse_float", builtin_parse_float);

        // Collection functions
        self.register("push", builtin_push);
        self.register("pop", builtin_pop);
        self.register("reverse", builtin_reverse);
        self.register("sort", builtin_sort);

        // Environment functions
        self.register("env_args", builtin_env_args);
        self.register("env_var", builtin_env_var);
        self.register("env_set_var", builtin_env_set_var);
        self.register("env_remove_var", builtin_env_remove_var);
        self.register("env_vars", builtin_env_vars);
        self.register("env_current_dir", builtin_env_current_dir);
        self.register("env_set_current_dir", builtin_env_set_current_dir);
        self.register("env_temp_dir", builtin_env_temp_dir);

        // File system functions
        self.register("fs_read", builtin_fs_read);
        self.register("fs_write", builtin_fs_write);
        self.register("fs_exists", builtin_fs_exists);
        self.register("fs_create_dir", builtin_fs_create_dir);
        self.register("fs_remove_file", builtin_fs_remove_file);
        self.register("fs_remove_dir", builtin_fs_remove_dir);
        self.register("fs_copy", builtin_fs_copy);
        self.register("fs_rename", builtin_fs_rename);
        self.register("fs_metadata", builtin_fs_metadata);
        self.register("fs_read_dir", builtin_fs_read_dir);
        self.register("fs_canonicalize", builtin_fs_canonicalize);
        self.register("fs_is_file", builtin_fs_is_file);

        // Path functions
        self.register("path_join", builtin_path_join);
        self.register("path_join_many", builtin_path_join_many);
        self.register("path_parent", builtin_path_parent);
        self.register("path_file_name", builtin_path_file_name);
        self.register("path_file_stem", builtin_path_file_stem);
        self.register("path_extension", builtin_path_extension);
        self.register("path_is_absolute", builtin_path_is_absolute);
        self.register("path_is_relative", builtin_path_is_relative);
        self.register("path_canonicalize", builtin_path_canonicalize);
        self.register("path_with_extension", builtin_path_with_extension);
        self.register("path_with_file_name", builtin_path_with_file_name);
        self.register("path_components", builtin_path_components);
        self.register("path_normalize", builtin_path_normalize);

        // JSON functions
        self.register("json_parse", builtin_json_parse);
        self.register("json_stringify", builtin_json_stringify);
        self.register("json_pretty", builtin_json_pretty);
        self.register("json_read", builtin_json_read);
        self.register("json_write", builtin_json_write);
        self.register("json_validate", builtin_json_validate);
        self.register("json_type", builtin_json_type);
        self.register("json_merge", builtin_json_merge);
        self.register("json_get", builtin_json_get);
        self.register("json_set", builtin_json_set);

        // HTTP functions (not available in WASM)
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.register("http_get", builtin_http_get);
            self.register("http_post", builtin_http_post);
            self.register("http_put", builtin_http_put);
            self.register("http_delete", builtin_http_delete);
        }
    }

    /// Register a builtin function
    fn register(&mut self, name: &str, func: BuiltinFunction) {
        self.functions.insert(name.to_string(), func);
    }

    /// Call a builtin function
    ///
    /// # Complexity
    /// Cyclomatic complexity: 2 (within limit of 10)
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value, InterpreterError> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        } else {
            Err(InterpreterError::RuntimeError(format!(
                "Unknown builtin function: {name}"
            )))
        }
    }

    /// Check if a function is a builtin
    pub fn is_builtin(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

// I/O Functions

/// Built-in println function (DEPRECATED - use `eval_builtin.rs` instead)
///
/// NOTE: This function is kept for backward compatibility but is not used
/// by the interpreter. The actual println implementation is in `eval_builtin.rs`
/// which properly captures output for notebook use.
///
/// # Complexity
/// Cyclomatic complexity: 2 (within limit of 10)
fn builtin_println(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.is_empty() {
        println!();
    } else {
        let output = args
            .iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
            .join(" ");
        println!("{output}");
    }
    Ok(Value::nil())
}

/// Built-in print function (DEPRECATED - use `eval_builtin.rs` instead)
///
/// NOTE: This function is kept for backward compatibility but is not used
/// by the interpreter. The actual print implementation is in `eval_builtin.rs`
///
/// # Complexity
/// Cyclomatic complexity: 1 (within limit of 10)
fn builtin_print(args: &[Value]) -> Result<Value, InterpreterError> {
    let output = args
        .iter()
        .map(|v| format!("{v}"))
        .collect::<Vec<_>>()
        .join(" ");
    print!("{output}");
    Ok(Value::nil())
}

/// Built-in debug print function
///
/// # Complexity
/// Cyclomatic complexity: 3 (within limit of 10)
fn builtin_dbg(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() == 1 {
        println!("[DEBUG] {:?}", args[0]);
        Ok(args[0].clone())
    } else {
        println!("[DEBUG] {args:?}");
        Ok(Value::from_array(args.to_vec()))
    }
}

// Type and Inspection Functions

/// Built-in len function
///
/// # Complexity
/// Cyclomatic complexity: 5 (within limit of 10)
fn builtin_len(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(format!(
            "len() expects exactly 1 argument, got {}",
            args.len()
        )));
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        Value::Object(fields) => Ok(Value::Integer(fields.len() as i64)),
        Value::Range { start, end, .. } => match (start.as_ref(), end.as_ref()) {
            (Value::Integer(s), Value::Integer(e)) => Ok(Value::Integer((e - s).abs())),
            _ => Err(InterpreterError::RuntimeError(
                "Range bounds must be integers for len()".to_string(),
            )),
        },
        _ => Err(InterpreterError::RuntimeError(format!(
            "len() not supported for {}",
            args[0].type_name()
        ))),
    }
}

/// Built-in `type_of` function
///
/// # Complexity
/// Cyclomatic complexity: 1 (within limit of 10)
fn builtin_type_of(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(format!(
            "type_of() expects exactly 1 argument, got {}",
            args.len()
        )));
    }
    Ok(Value::from_string(args[0].type_name().to_string()))
}

/// Built-in `is_nil` function
///
/// # Complexity
/// Cyclomatic complexity: 2 (within limit of 10)
fn builtin_is_nil(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(format!(
            "is_nil() expects exactly 1 argument, got {}",
            args.len()
        )));
    }
    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}

// Math Functions

/// Built-in sqrt function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within limit of 10)
fn builtin_sqrt(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "sqrt() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Float((*n as f64).sqrt())),
        Value::Float(f) => Ok(Value::Float(f.sqrt())),
        _ => Err(InterpreterError::RuntimeError(
            "sqrt() expects a number".to_string(),
        )),
    }
}

/// Built-in pow function
///
/// # Complexity
/// Cyclomatic complexity: 6 (within limit of 10)
fn builtin_pow(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "pow() expects exactly 2 arguments".to_string(),
        ));
    }
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
            "pow() expects numeric arguments".to_string(),
        )),
    }
}

/// Built-in abs function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within limit of 10)
fn builtin_abs(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "abs() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(n.abs())),
        Value::Float(f) => Ok(Value::Float(f.abs())),
        _ => Err(InterpreterError::RuntimeError(
            "abs() expects a number".to_string(),
        )),
    }
}

/// Helper: Compare two values for min operation
/// Returns true if b < a
fn compare_less(a: &Value, b: &Value) -> Result<bool, InterpreterError> {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Ok(y < x),
        (Value::Float(x), Value::Float(y)) => Ok(y < x),
        _ => Err(InterpreterError::RuntimeError(
            "min() expects all arguments to be numbers of the same type".to_string(),
        )),
    }
}

/// Built-in min function
///
/// # Complexity
/// Cyclomatic complexity: 2 (reduced via helper extraction)
fn builtin_min(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "min() expects at least 1 argument".to_string(),
        ));
    }

    let mut min_val = &args[0];
    for arg in &args[1..] {
        if compare_less(min_val, arg)? {
            min_val = arg;
        }
    }
    Ok(min_val.clone())
}

/// Helper: Compare two values for max operation
/// Returns true if b > a
fn compare_greater(a: &Value, b: &Value) -> Result<bool, InterpreterError> {
    match (a, b) {
        (Value::Integer(x), Value::Integer(y)) => Ok(y > x),
        (Value::Float(x), Value::Float(y)) => Ok(y > x),
        _ => Err(InterpreterError::RuntimeError(
            "max() expects all arguments to be numbers of the same type".to_string(),
        )),
    }
}

/// Built-in max function
///
/// # Complexity
/// Cyclomatic complexity: 2 (reduced via helper extraction)
fn builtin_max(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "max() expects at least 1 argument".to_string(),
        ));
    }

    let mut max_val = &args[0];
    for arg in &args[1..] {
        if compare_greater(max_val, arg)? {
            max_val = arg;
        }
    }
    Ok(max_val.clone())
}

/// Built-in floor function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within limit of 10)
fn builtin_floor(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "floor() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(f.floor() as i64)),
        _ => Err(InterpreterError::RuntimeError(
            "floor() expects a number".to_string(),
        )),
    }
}

/// Built-in ceil function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within limit of 10)
fn builtin_ceil(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "ceil() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(f.ceil() as i64)),
        _ => Err(InterpreterError::RuntimeError(
            "ceil() expects a number".to_string(),
        )),
    }
}

/// Built-in round function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within limit of 10)
fn builtin_round(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "round() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Integer(n) => Ok(Value::Integer(*n)),
        Value::Float(f) => Ok(Value::Integer(f.round() as i64)),
        _ => Err(InterpreterError::RuntimeError(
            "round() expects a number".to_string(),
        )),
    }
}

// String Functions

/// Built-in `to_string` function
///
/// # Complexity
/// Cyclomatic complexity: 1 (within limit of 10)
fn builtin_to_string(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(format!(
            "to_string() expects exactly 1 argument, got {}",
            args.len()
        )));
    }
    Ok(Value::from_string(format!("{}", args[0])))
}

/// Built-in `parse_int` function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within limit of 10)
fn builtin_parse_int(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "parse_int() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::String(s) => s
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| InterpreterError::RuntimeError(format!("Cannot parse '{s}' as integer"))),
        Value::Integer(n) => Ok(Value::Integer(*n)),
        _ => Err(InterpreterError::RuntimeError(
            "parse_int() expects a string".to_string(),
        )),
    }
}

/// Built-in `parse_float` function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within limit of 10)
fn builtin_parse_float(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "parse_float() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::String(s) => s
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| InterpreterError::RuntimeError(format!("Cannot parse '{s}' as float"))),
        Value::Float(f) => Ok(Value::Float(*f)),
        Value::Integer(n) => Ok(Value::Float(*n as f64)),
        _ => Err(InterpreterError::RuntimeError(
            "parse_float() expects a string".to_string(),
        )),
    }
}

// Collection Functions

/// Built-in push function
///
/// # Complexity
/// Cyclomatic complexity: 3 (within limit of 10)
fn builtin_push(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "push() expects exactly 2 arguments".to_string(),
        ));
    }
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

/// Built-in pop function
///
/// # Complexity
/// Cyclomatic complexity: 4 (within limit of 10)
fn builtin_pop(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "pop() expects exactly 1 argument".to_string(),
        ));
    }
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

/// Built-in reverse function
///
/// # Complexity
/// Cyclomatic complexity: 3 (within limit of 10)
fn builtin_reverse(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "reverse() expects exactly 1 argument".to_string(),
        ));
    }
    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.to_vec();
            new_arr.reverse();
            Ok(Value::from_array(new_arr))
        }
        _ => Err(InterpreterError::RuntimeError(
            "reverse() expects an array".to_string(),
        )),
    }
}

/// Built-in sort function
///
/// # Complexity
/// Cyclomatic complexity: 6 (within limit of 10)
fn builtin_sort(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "sort() expects exactly 1 argument".to_string(),
        ));
    }
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

// Environment Functions

/// Built-in `env_args` function - returns command-line arguments
///
/// # Examples
///
/// ```
/// use ruchy::runtime::builtins::BuiltinRegistry;
/// use ruchy::runtime::Value;
///
/// let registry = BuiltinRegistry::new();
/// let result = registry.call("env_args", &[]).unwrap();
/// // Returns array of command-line arguments
/// ```
///
/// # Complexity
/// Cyclomatic complexity: 1 (within limit of 10)
fn builtin_env_args(args: &[Value]) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "env_args() expects no arguments".to_string(),
        ));
    }

    // Get command-line arguments
    let cmd_args: Vec<Value> = std::env::args()
        .map(|s| Value::String(s.into()))
        .collect();

    Ok(Value::from_array(cmd_args))
}

// Get environment variable by key
fn builtin_env_var(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "env_var() expects 1 argument".to_string(),
        ));
    }

    match &args[0] {
        Value::String(key) => match std::env::var(key.as_ref()) {
            Ok(val) => Ok(Value::from_string(val)),
            Err(_) => Err(InterpreterError::RuntimeError(
                format!("Environment variable '{key}' not found"),
            )),
        },
        _ => Err(InterpreterError::RuntimeError(
            "env_var() expects a string argument".to_string(),
        )),
    }
}

// Set environment variable
// Complexity: 3 (within Toyota Way limits)
fn builtin_env_set_var(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError(
            "env_set_var() expects 2 arguments".to_string(),
        ));
    }

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

// Remove environment variable
// Complexity: 2 (within Toyota Way limits)
fn builtin_env_remove_var(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "env_remove_var() expects 1 argument".to_string(),
        ));
    }

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

// Get all environment variables
// Complexity: 1 (within Toyota Way limits)
fn builtin_env_vars(args: &[Value]) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "env_vars() expects no arguments".to_string(),
        ));
    }

    let vars: HashMap<String, Value> = std::env::vars()
        .map(|(k, v)| (k, Value::from_string(v)))
        .collect();

    Ok(Value::Object(Arc::new(vars)))
}

// Get current working directory
// Complexity: 2 (within Toyota Way limits)
fn builtin_env_current_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "env_current_dir() expects no arguments".to_string(),
        ));
    }

    match std::env::current_dir() {
        Ok(path) => Ok(Value::from_string(path.to_string_lossy().to_string())),
        Err(e) => Err(InterpreterError::RuntimeError(
            format!("Failed to get current directory: {e}"),
        )),
    }
}

// Set current working directory
// Complexity: 2 (within Toyota Way limits)
fn builtin_env_set_current_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError(
            "env_set_current_dir() expects 1 argument".to_string(),
        ));
    }

    match &args[0] {
        Value::String(path) => match std::env::set_current_dir(path.as_ref()) {
            Ok(()) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(
                format!("Failed to set current directory: {e}"),
            )),
        },
        _ => Err(InterpreterError::RuntimeError(
            "env_set_current_dir() expects a string argument".to_string(),
        )),
    }
}

// Get system temp directory
// Complexity: 1 (within Toyota Way limits)
fn builtin_env_temp_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    if !args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "env_temp_dir() expects no arguments".to_string(),
        ));
    }

    let temp = std::env::temp_dir();
    Ok(Value::from_string(temp.to_string_lossy().to_string()))
}

// File System Functions

// Read file contents
fn builtin_fs_read(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("fs_read() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => match std::fs::read_to_string(path.as_ref()) {
            Ok(content) => Ok(Value::from_string(content)),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to read file: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_read() expects a string argument".to_string())),
    }
}

// Write to file
fn builtin_fs_write(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("fs_write() expects 2 arguments".to_string()));
    }
    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(content)) => match std::fs::write(path.as_ref(), content.as_ref()) {
            Ok(()) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to write file: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_write() expects two string arguments".to_string())),
    }
}

// Check if path exists
fn builtin_fs_exists(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("fs_exists() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => Ok(Value::Bool(std::path::Path::new(path.as_ref()).exists())),
        _ => Err(InterpreterError::RuntimeError("fs_exists() expects a string argument".to_string())),
    }
}

// Create directory
fn builtin_fs_create_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("fs_create_dir() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => match std::fs::create_dir(path.as_ref()) {
            Ok(()) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to create directory: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_create_dir() expects a string argument".to_string())),
    }
}

// Remove file
fn builtin_fs_remove_file(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("fs_remove_file() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => match std::fs::remove_file(path.as_ref()) {
            Ok(()) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to remove file: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_remove_file() expects a string argument".to_string())),
    }
}

// Remove directory
fn builtin_fs_remove_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("fs_remove_dir() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => match std::fs::remove_dir(path.as_ref()) {
            Ok(()) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to remove directory: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_remove_dir() expects a string argument".to_string())),
    }
}

// Copy file
fn builtin_fs_copy(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("fs_copy() expects 2 arguments".to_string()));
    }
    match (&args[0], &args[1]) {
        (Value::String(from), Value::String(to)) => match std::fs::copy(from.as_ref(), to.as_ref()) {
            Ok(_) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to copy file: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_copy() expects two string arguments".to_string())),
    }
}

// Rename/move file
fn builtin_fs_rename(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("fs_rename() expects 2 arguments".to_string()));
    }
    match (&args[0], &args[1]) {
        (Value::String(from), Value::String(to)) => match std::fs::rename(from.as_ref(), to.as_ref()) {
            Ok(()) => Ok(Value::Nil),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to rename file: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_rename() expects two string arguments".to_string())),
    }
}

// Get file metadata (returns object with size, is_dir, is_file)
fn builtin_fs_metadata(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("fs_metadata() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => match std::fs::metadata(path.as_ref()) {
            Ok(meta) => {
                let mut map = HashMap::new();
                map.insert("size".to_string(), Value::Integer(meta.len() as i64));
                map.insert("is_dir".to_string(), Value::Bool(meta.is_dir()));
                map.insert("is_file".to_string(), Value::Bool(meta.is_file()));
                Ok(Value::Object(Arc::new(map)))
            },
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to get metadata: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_metadata() expects a string argument".to_string())),
    }
}

// Read directory contents
fn builtin_fs_read_dir(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("fs_read_dir() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => match std::fs::read_dir(path.as_ref()) {
            Ok(entries) => {
                let names: Result<Vec<Value>, _> = entries
                    .map(|e| e.map(|entry| Value::from_string(entry.file_name().to_string_lossy().to_string())))
                    .collect();
                match names {
                    Ok(vec) => Ok(Value::from_array(vec)),
                    Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to read directory: {e}"))),
                }
            },
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to read directory: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_read_dir() expects a string argument".to_string())),
    }
}

// Canonicalize path (get absolute path)
fn builtin_fs_canonicalize(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("fs_canonicalize() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => match std::fs::canonicalize(path.as_ref()) {
            Ok(canonical) => Ok(Value::from_string(canonical.to_string_lossy().to_string())),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to canonicalize path: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("fs_canonicalize() expects a string argument".to_string())),
    }
}

// Check if path is a file
fn builtin_fs_is_file(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("fs_is_file() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => Ok(Value::Bool(std::path::Path::new(path.as_ref()).is_file())),
        _ => Err(InterpreterError::RuntimeError("fs_is_file() expects a string argument".to_string())),
    }
}

// ==================== PATH FUNCTIONS ====================
// Phase 3: STDLIB_ACCESS_PLAN - Path Module (13 functions)

fn builtin_path_join(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("path_join() expects 2 arguments".to_string()));
    }
    match (&args[0], &args[1]) {
        (Value::String(base), Value::String(component)) => {
            let path = std::path::Path::new(base.as_ref()).join(component.as_ref());
            Ok(Value::from_string(path.to_string_lossy().to_string()))
        },
        _ => Err(InterpreterError::RuntimeError("path_join() expects two string arguments".to_string())),
    }
}

fn builtin_path_join_many(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_join_many() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::Array(components) => {
            let path = build_path_from_components(components)?;
            Ok(Value::from_string(path.to_string_lossy().to_string()))
        },
        _ => Err(InterpreterError::RuntimeError("path_join_many() expects an array argument".to_string())),
    }
}

/// Helper: Build `PathBuf` from Value array components
/// Complexity: 3 (reduced cognitive load)
fn build_path_from_components(components: &[Value]) -> Result<std::path::PathBuf, InterpreterError> {
    let mut path = std::path::PathBuf::new();
    for component in components {
        match component {
            Value::String(s) => path.push(s.as_ref()),
            _ => return Err(InterpreterError::RuntimeError("path_join_many() expects array of strings".to_string())),
        }
    }
    Ok(path)
}

fn builtin_path_parent(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_parent() expects 1 argument".to_string()));
    }
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

fn builtin_path_file_name(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_file_name() expects 1 argument".to_string()));
    }
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

fn builtin_path_file_stem(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_file_stem() expects 1 argument".to_string()));
    }
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

fn builtin_path_extension(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_extension() expects 1 argument".to_string()));
    }
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

fn builtin_path_is_absolute(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_is_absolute() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            Ok(Value::Bool(p.is_absolute()))
        },
        _ => Err(InterpreterError::RuntimeError("path_is_absolute() expects a string argument".to_string())),
    }
}

fn builtin_path_is_relative(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_is_relative() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            Ok(Value::Bool(p.is_relative()))
        },
        _ => Err(InterpreterError::RuntimeError("path_is_relative() expects a string argument".to_string())),
    }
}

fn builtin_path_canonicalize(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_canonicalize() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => match std::fs::canonicalize(path.as_ref()) {
            Ok(canonical) => Ok(Value::from_string(canonical.to_string_lossy().to_string())),
            Err(e) => Err(InterpreterError::RuntimeError(format!("Failed to canonicalize path: {e}"))),
        },
        _ => Err(InterpreterError::RuntimeError("path_canonicalize() expects a string argument".to_string())),
    }
}

fn builtin_path_with_extension(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("path_with_extension() expects 2 arguments".to_string()));
    }
    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(ext)) => {
            let p = std::path::Path::new(path.as_ref()).with_extension(ext.as_ref());
            Ok(Value::from_string(p.to_string_lossy().to_string()))
        },
        _ => Err(InterpreterError::RuntimeError("path_with_extension() expects two string arguments".to_string())),
    }
}

fn builtin_path_with_file_name(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("path_with_file_name() expects 2 arguments".to_string()));
    }
    match (&args[0], &args[1]) {
        (Value::String(path), Value::String(name)) => {
            let p = std::path::Path::new(path.as_ref()).with_file_name(name.as_ref());
            Ok(Value::from_string(p.to_string_lossy().to_string()))
        },
        _ => Err(InterpreterError::RuntimeError("path_with_file_name() expects two string arguments".to_string())),
    }
}

fn builtin_path_components(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_components() expects 1 argument".to_string()));
    }
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

fn builtin_path_normalize(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("path_normalize() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => {
            let p = std::path::Path::new(path.as_ref());
            // Simple normalization: remove "." and resolve ".."
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

// ==================== JSON FUNCTIONS ====================
// Phase 4: STDLIB_ACCESS_PLAN - JSON Module (10 functions)
// Thin wrapper pattern: delegate to serde_json (complexity ≤2 per function)

/// Convert `serde_json::Value` to Ruchy Value
/// Complexity: 2 (thin wrapper helper)
fn json_value_to_ruchy(json: serde_json::Value) -> Value {
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
            let values: Vec<Value> = arr.into_iter().map(json_value_to_ruchy).collect();
            Value::Array(values.into())
        },
        serde_json::Value::Object(obj) => {
            let mut map = std::collections::HashMap::new();
            for (k, v) in obj {
                map.insert(k, json_value_to_ruchy(v));
            }
            Value::Object(std::sync::Arc::new(map))
        },
    }
}

/// Convert Ruchy Value to `serde_json::Value`
/// Complexity: 2 (thin wrapper helper)
/// Convert Ruchy value to JSON value
/// Complexity: 5 (reduced by extracting object conversion helpers)
fn ruchy_value_to_json(value: &Value) -> Result<serde_json::Value, InterpreterError> {
    match value {
        Value::Nil => Ok(serde_json::Value::Null),
        Value::Bool(b) => Ok(serde_json::Value::Bool(*b)),
        Value::Integer(i) => Ok(serde_json::json!(*i)),
        Value::Float(f) => Ok(serde_json::json!(*f)),
        Value::String(s) => Ok(serde_json::Value::String(s.to_string())),
        Value::Array(arr) => convert_array_to_json(arr),
        Value::Object(map) => convert_object_to_json(map),
        Value::ObjectMut(map) => convert_object_mut_to_json(map),
        _ => Err(InterpreterError::RuntimeError(format!("Cannot convert {value:?} to JSON"))),
    }
}

/// Convert Ruchy array to JSON array
/// Complexity: 2 (simple map + collect)
fn convert_array_to_json(arr: &[Value]) -> Result<serde_json::Value, InterpreterError> {
    let json_arr: Result<Vec<serde_json::Value>, _> = arr.iter()
        .map(ruchy_value_to_json)
        .collect();
    Ok(serde_json::Value::Array(json_arr?))
}

/// Convert immutable Ruchy object to JSON object
/// Complexity: 3 (iteration + recursive conversion)
fn convert_object_to_json(map: &std::collections::HashMap<String, Value>) -> Result<serde_json::Value, InterpreterError> {
    let mut json_obj = serde_json::Map::new();
    for (k, v) in map {
        json_obj.insert(k.clone(), ruchy_value_to_json(v)?);
    }
    Ok(serde_json::Value::Object(json_obj))
}

/// Convert mutable Ruchy object to JSON object
/// Complexity: 3 (lock + iteration + recursive conversion)
fn convert_object_mut_to_json(map: &std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Value>>>) -> Result<serde_json::Value, InterpreterError> {
    let guard = map.lock().unwrap();
    let mut json_obj = serde_json::Map::new();
    for (k, v) in guard.iter() {
        json_obj.insert(k.clone(), ruchy_value_to_json(v)?);
    }
    Ok(serde_json::Value::Object(json_obj))
}

/// `json_parse(str)` - Parse JSON string to Ruchy value
/// Complexity: 2 (thin wrapper)
fn builtin_json_parse(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("json_parse() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(s) => {
            match serde_json::from_str::<serde_json::Value>(s) {
                Ok(json) => Ok(json_value_to_ruchy(json)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("JSON parse error: {e}"))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("json_parse() expects a string argument".to_string())),
    }
}

/// `json_stringify(value)` - Convert Ruchy value to JSON string
/// Complexity: 2 (thin wrapper)
fn builtin_json_stringify(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("json_stringify() expects 1 argument".to_string()));
    }
    let json = ruchy_value_to_json(&args[0])?;
    match serde_json::to_string(&json) {
        Ok(s) => Ok(Value::from_string(s)),
        Err(e) => Err(InterpreterError::RuntimeError(format!("JSON stringify error: {e}"))),
    }
}

/// `json_pretty(value)` - Pretty-print JSON with indentation
/// Complexity: 2 (thin wrapper)
fn builtin_json_pretty(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("json_pretty() expects 1 argument".to_string()));
    }
    let json = ruchy_value_to_json(&args[0])?;
    match serde_json::to_string_pretty(&json) {
        Ok(s) => Ok(Value::from_string(s)),
        Err(e) => Err(InterpreterError::RuntimeError(format!("JSON pretty error: {e}"))),
    }
}

/// `json_read(path)` - Read and parse JSON file
/// Complexity: 2 (thin wrapper)
fn builtin_json_read(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("json_read() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(path) => {
            let content = std::fs::read_to_string(path.as_ref())
                .map_err(|e| InterpreterError::RuntimeError(format!("Failed to read file: {e}")))?;
            match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => Ok(json_value_to_ruchy(json)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("JSON parse error: {e}"))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("json_read() expects a string argument".to_string())),
    }
}

/// `json_write(path`, value) - Write value as JSON to file
/// Complexity: 2 (thin wrapper)
/// `json_write(path`, value) - Write Ruchy value as JSON file
/// Complexity: 3 (reduced by extracting serialization)
fn builtin_json_write(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("json_write() expects 2 arguments".to_string()));
    }
    match &args[0] {
        Value::String(path) => {
            let content = serialize_value_to_json_string(&args[1])?;
            write_json_to_file(path, &content)?;
            Ok(Value::Bool(true))
        },
        _ => Err(InterpreterError::RuntimeError("json_write() expects first argument to be string".to_string())),
    }
}

/// Serialize Ruchy value to JSON string
/// Complexity: 2 (conversion + stringify)
fn serialize_value_to_json_string(value: &Value) -> Result<String, InterpreterError> {
    let json = ruchy_value_to_json(value)?;
    serde_json::to_string_pretty(&json)
        .map_err(|e| InterpreterError::RuntimeError(format!("JSON stringify error: {e}")))
}

/// Write JSON string to file
/// Complexity: 2 (file write with error handling)
fn write_json_to_file(path: &str, content: &str) -> Result<(), InterpreterError> {
    std::fs::write(path, content)
        .map_err(|e| InterpreterError::RuntimeError(format!("Failed to write file: {e}")))
}

/// `json_validate(str)` - Check if string is valid JSON
/// Complexity: 2 (thin wrapper)
fn builtin_json_validate(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("json_validate() expects 1 argument".to_string()));
    }
    match &args[0] {
        Value::String(s) => {
            let is_valid = serde_json::from_str::<serde_json::Value>(s).is_ok();
            Ok(Value::Bool(is_valid))
        },
        _ => Err(InterpreterError::RuntimeError("json_validate() expects a string argument".to_string())),
    }
}

/// `json_type(str)` - Get JSON type without full parsing
/// Complexity: 2 (thin wrapper)
fn builtin_json_type(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("json_type() expects 1 argument".to_string()));
    }
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
                Err(e) => Err(InterpreterError::RuntimeError(format!("JSON parse error: {e}"))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("json_type() expects a string argument".to_string())),
    }
}

/// `json_merge(obj1`, obj2) - Deep merge two JSON objects
/// Complexity: 2 (delegates to helper)
fn builtin_json_merge(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("json_merge() expects 2 arguments".to_string()));
    }

    // Convert both args to JSON, merge, convert back
    let json1 = ruchy_value_to_json(&args[0])?;
    let json2 = ruchy_value_to_json(&args[1])?;

    let merged = merge_json_values(json1, json2);
    Ok(json_value_to_ruchy(merged))
}

/// Helper: Deep merge two JSON values
/// Complexity: 3 (recursive merge logic)
fn merge_json_values(a: serde_json::Value, b: serde_json::Value) -> serde_json::Value {
    match (a, b) {
        (serde_json::Value::Object(mut a_map), serde_json::Value::Object(b_map)) => {
            for (k, v) in b_map {
                if let Some(a_val) = a_map.get_mut(&k) {
                    *a_val = merge_json_values(a_val.clone(), v);
                } else {
                    a_map.insert(k, v);
                }
            }
            serde_json::Value::Object(a_map)
        },
        (_, b_val) => b_val,
    }
}

/// `json_get(obj`, path) - Get nested value by path (e.g., "user.name")
/// Complexity: 2 (thin wrapper)
fn builtin_json_get(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("json_get() expects 2 arguments".to_string()));
    }

    let json = ruchy_value_to_json(&args[0])?;

    match &args[1] {
        Value::String(path) => {
            let parts: Vec<&str> = path.split('.').collect();
            let result = get_json_path(&json, &parts);
            match result {
                Some(val) => Ok(json_value_to_ruchy(val.clone())),
                None => Ok(Value::Nil),
            }
        },
        _ => Err(InterpreterError::RuntimeError("json_get() expects second argument to be string".to_string())),
    }
}

/// Helper: Get value at JSON path
/// Complexity: 2 (recursive path traversal)
fn get_json_path<'a>(json: &'a serde_json::Value, path: &[&str]) -> Option<&'a serde_json::Value> {
    if path.is_empty() {
        return Some(json);
    }

    match json {
        serde_json::Value::Object(map) => {
            map.get(path[0]).and_then(|v| get_json_path(v, &path[1..]))
        },
        _ => None,
    }
}

/// `json_set(obj`, path, value) - Set nested value by path
/// Complexity: 2 (thin wrapper)
fn builtin_json_set(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 3 {
        return Err(InterpreterError::RuntimeError("json_set() expects 3 arguments".to_string()));
    }

    let mut json = ruchy_value_to_json(&args[0])?;
    let new_value = ruchy_value_to_json(&args[2])?;

    match &args[1] {
        Value::String(path) => {
            let parts: Vec<&str> = path.split('.').collect();
            set_json_path(&mut json, &parts, new_value);
            Ok(json_value_to_ruchy(json))
        },
        _ => Err(InterpreterError::RuntimeError("json_set() expects second argument to be string".to_string())),
    }
}

/// Helper: Set value at JSON path
/// Complexity: 3 (recursive path setting with mutation)
/// Set value at JSON path (recursive)
/// Complexity: 4 (reduced by extracting single-level setter)
fn set_json_path(json: &mut serde_json::Value, path: &[&str], value: serde_json::Value) {
    if path.is_empty() {
        *json = value;
        return;
    }

    if path.len() == 1 {
        set_json_value_at_key(json, path[0], value);
    } else {
        set_json_path_recursive(json, path, value);
    }
}

/// Set JSON value at single key
/// Complexity: 2 (single-level operation)
fn set_json_value_at_key(json: &mut serde_json::Value, key: &str, value: serde_json::Value) {
    if let serde_json::Value::Object(map) = json {
        map.insert(key.to_string(), value);
    }
}

/// Set JSON value at nested path
/// Complexity: 3 (recursive path traversal)
fn set_json_path_recursive(json: &mut serde_json::Value, path: &[&str], value: serde_json::Value) {
    if let serde_json::Value::Object(map) = json {
        if let Some(next) = map.get_mut(path[0]) {
            set_json_path(next, &path[1..], value);
        }
    }
}

// ==============================================================================
// HTTP Builtin Functions (STDLIB-PHASE-5)
// ==============================================================================
// Thin wrappers around crate::stdlib::http module
// Complexity: ≤2 per function (Toyota Way limits)

/// Builtin: `http_get(url)`
/// Sends GET request and returns response body as string
/// Complexity: 2 (error handling + stdlib delegation)
/// Note: Not available for WASM targets
#[cfg(not(target_arch = "wasm32"))]
fn builtin_http_get(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("http_get() expects 1 argument".to_string()));
    }

    match &args[0] {
        Value::String(url) => {
            match crate::stdlib::http::get(url) {
                Ok(response) => Ok(Value::from_string(response)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("HTTP GET failed: {e}"))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("http_get() expects a string URL".to_string())),
    }
}

/// Builtin: `http_post(url`, body)
/// Sends POST request with body and returns response
/// Complexity: 2 (error handling + stdlib delegation)
/// Note: Not available for WASM targets
#[cfg(not(target_arch = "wasm32"))]
fn builtin_http_post(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("http_post() expects 2 arguments".to_string()));
    }

    match (&args[0], &args[1]) {
        (Value::String(url), Value::String(body)) => {
            match crate::stdlib::http::post(url, body) {
                Ok(response) => Ok(Value::from_string(response)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("HTTP POST failed: {e}"))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("http_post() expects two string arguments".to_string())),
    }
}

/// Builtin: `http_put(url`, body)
/// Sends PUT request with body and returns response
/// Complexity: 2 (error handling + stdlib delegation)
/// Note: Not available for WASM targets
#[cfg(not(target_arch = "wasm32"))]
fn builtin_http_put(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 2 {
        return Err(InterpreterError::RuntimeError("http_put() expects 2 arguments".to_string()));
    }

    match (&args[0], &args[1]) {
        (Value::String(url), Value::String(body)) => {
            match crate::stdlib::http::put(url, body) {
                Ok(response) => Ok(Value::from_string(response)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("HTTP PUT failed: {e}"))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("http_put() expects two string arguments".to_string())),
    }
}

/// Builtin: `http_delete(url)`
/// Sends DELETE request and returns response
/// Complexity: 2 (error handling + stdlib delegation)
/// Note: Not available for WASM targets
#[cfg(not(target_arch = "wasm32"))]
fn builtin_http_delete(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() != 1 {
        return Err(InterpreterError::RuntimeError("http_delete() expects 1 argument".to_string()));
    }

    match &args[0] {
        Value::String(url) => {
            match crate::stdlib::http::delete(url) {
                Ok(response) => Ok(Value::from_string(response)),
                Err(e) => Err(InterpreterError::RuntimeError(format!("HTTP DELETE failed: {e}"))),
            }
        },
        _ => Err(InterpreterError::RuntimeError("http_delete() expects a string URL".to_string())),
    }
}

/// Built-in `assert_eq` function for testing
/// Panics if the two values are not equal
///
/// # Arguments
/// * args[0] - Expected value
/// * args[1] - Actual value
/// * args[2] - Optional message (string)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within limit of 10)
fn builtin_assert_eq(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.len() < 2 {
        return Err(InterpreterError::RuntimeError(
            "assert_eq() expects at least 2 arguments (expected, actual)".to_string(),
        ));
    }

    let expected = &args[0];
    let actual = &args[1];
    let message = if args.len() > 2 {
        format!("{}", args[2])
    } else {
        format!("Assertion failed: expected {expected:?}, got {actual:?}")
    };

    if expected == actual {
        Ok(Value::nil())
    } else {
        Err(InterpreterError::AssertionFailed(message))
    }
}

/// Built-in assert function for testing
/// Panics if the condition is false
///
/// # Arguments
/// * args[0] - Condition (must be boolean)
/// * args[1] - Optional message (string)
///
/// # Complexity
/// Cyclomatic complexity: 3 (within limit of 10)
fn builtin_assert(args: &[Value]) -> Result<Value, InterpreterError> {
    if args.is_empty() {
        return Err(InterpreterError::RuntimeError(
            "assert() expects at least 1 argument (condition)".to_string(),
        ));
    }

    let condition = &args[0];
    let message = if args.len() > 1 {
        format!("{}", args[1])
    } else {
        "Assertion failed".to_string()
    };

    match condition {
        Value::Bool(true) => Ok(Value::nil()),
        Value::Bool(false) => Err(InterpreterError::AssertionFailed(message)),
        _ => Err(InterpreterError::RuntimeError(
            "assert() expects a boolean condition".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_println() {
        let result = builtin_println(&[Value::from_string("test".to_string())]).unwrap();
        assert_eq!(result, Value::nil());
    }

    #[test]
    fn test_builtin_len() {
        let result = builtin_len(&[Value::from_string("hello".to_string())]).unwrap();
        assert_eq!(result, Value::Integer(5));

        let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
        let result = builtin_len(&[arr]).unwrap();
        assert_eq!(result, Value::Integer(2));
    }

    #[test]
    fn test_builtin_type_of() {
        let result = builtin_type_of(&[Value::Integer(42)]).unwrap();
        assert_eq!(result, Value::from_string("integer".to_string()));
    }

    #[test]
    fn test_builtin_sqrt() {
        let result = builtin_sqrt(&[Value::Integer(9)]).unwrap();
        assert_eq!(result, Value::Float(3.0));
    }

    #[test]
    fn test_builtin_abs() {
        let result = builtin_abs(&[Value::Integer(-42)]).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_builtin_registry() {
        let registry = BuiltinRegistry::new();
        assert!(registry.is_builtin("println"));
        assert!(registry.is_builtin("len"));
        assert!(!registry.is_builtin("not_a_builtin"));
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_abs_idempotent(n: i64) {
            let val = Value::Integer(n);
            let result1 = builtin_abs(&[val]).unwrap();
            let result2 = builtin_abs(&[result1.clone()]).unwrap();
            prop_assert_eq!(result1, result2);
        }

        #[test]
        fn test_min_max_consistency(a: i64, b: i64) {
            let min_result = builtin_min(&[Value::Integer(a), Value::Integer(b)]).unwrap();
            let max_result = builtin_max(&[Value::Integer(a), Value::Integer(b)]).unwrap();

            // min and max should return one of the inputs
            prop_assert!(min_result == Value::Integer(a) || min_result == Value::Integer(b));
            prop_assert!(max_result == Value::Integer(a) || max_result == Value::Integer(b));

            // min should be <= max
            match (min_result, max_result) {
                (Value::Integer(min), Value::Integer(max)) => prop_assert!(min <= max),
                _ => prop_assert!(false),
            }
        }

        #[test]
        fn test_to_string_parse_roundtrip(n: i64) {
            let val = Value::Integer(n);
            let str_val = builtin_to_string(&[val]).unwrap();
            let parsed = builtin_parse_int(&[str_val]).unwrap();
            prop_assert_eq!(parsed, Value::Integer(n));
        }
    }
}
