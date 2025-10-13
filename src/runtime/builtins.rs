//! Built-in functions module
//!
//! This module contains all built-in functions like println, len, `type_of`, etc.
//! Extracted from the monolithic interpreter.rs to improve maintainability.
//! Complexity: <10 per function (Toyota Way compliant)

use crate::runtime::{InterpreterError, Value};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

#[cfg(test)]
use std::sync::Arc;

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

/// Built-in env_args function - returns command-line arguments
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
                format!("Environment variable '{}' not found", key),
            )),
        },
        _ => Err(InterpreterError::RuntimeError(
            "env_var() expects a string argument".to_string(),
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
