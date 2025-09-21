//! Builtin Functions Initialization Module
//!
//! EXTREME TDD: Full test coverage, zero entropy, <10 complexity per function
//! Extracted from interpreter.rs to eliminate builtin initialization bloat.
//!
//! This module handles the initialization of builtin functions and constants
//! in the global environment. All builtins are registered as special string
//! markers that are handled during function call evaluation.

use crate::runtime::Value;
use std::collections::HashMap;

/// Initialize global environment with all builtin functions and constants
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
pub fn init_global_environment() -> HashMap<String, Value> {
    let mut global_env = HashMap::new();

    // Add builtin constants
    add_builtin_constants(&mut global_env);

    // Add all builtin function categories
    add_basic_builtins(&mut global_env);
    add_math_functions(&mut global_env);
    add_io_functions(&mut global_env);
    add_utility_functions(&mut global_env);
    add_type_conversion_functions(&mut global_env);
    add_advanced_utility_functions(&mut global_env);
    add_string_functions(&mut global_env);
    add_random_time_functions(&mut global_env);

    global_env
}

/// Add builtin constants to the environment
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_builtin_constants(global_env: &mut HashMap<String, Value>) {
    global_env.insert("nil".to_string(), Value::Nil);
}

/// Add basic builtin functions (format, `HashMap`, `DataFrame`)
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_basic_builtins(global_env: &mut HashMap<String, Value>) {
    global_env.insert(
        "format".to_string(),
        Value::from_string("__builtin_format__".to_string()),
    );
    global_env.insert(
        "HashMap".to_string(),
        Value::from_string("__builtin_hashmap__".to_string()),
    );
    global_env.insert(
        "DataFrame".to_string(),
        Value::from_string("__builtin_dataframe__".to_string()),
    );
    global_env.insert(
        "DataFrame::from_range".to_string(),
        Value::from_string("__builtin_dataframe_from_range__".to_string()),
    );
    global_env.insert(
        "DataFrame::from_rows".to_string(),
        Value::from_string("__builtin_dataframe_from_rows__".to_string()),
    );
    global_env.insert(
        "col".to_string(),
        Value::from_string("__builtin_col__".to_string()),
    );
}

/// Add math standard library functions
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_math_functions(global_env: &mut HashMap<String, Value>) {
    let math_functions = [
        "sqrt", "pow", "abs", "min", "max", "floor", "ceil", "round", "sin", "cos", "tan",
    ];

    for func_name in &math_functions {
        let builtin_name = format!("__builtin_{func_name}__");
        global_env.insert((*func_name).to_string(), Value::from_string(builtin_name));
    }
}

/// Add I/O and output functions
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_io_functions(global_env: &mut HashMap<String, Value>) {
    global_env.insert(
        "println".to_string(),
        Value::from_string("__builtin_println__".to_string()),
    );
    global_env.insert(
        "print".to_string(),
        Value::from_string("__builtin_print__".to_string()),
    );
    global_env.insert(
        "dbg".to_string(),
        Value::from_string("__builtin_dbg__".to_string()),
    );
}

/// Add basic utility functions
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_utility_functions(global_env: &mut HashMap<String, Value>) {
    global_env.insert(
        "len".to_string(),
        Value::from_string("__builtin_len__".to_string()),
    );
    global_env.insert(
        "range".to_string(),
        Value::from_string("__builtin_range__".to_string()),
    );
    global_env.insert(
        "typeof".to_string(),
        Value::from_string("__builtin_type__".to_string()),
    );
}

/// Add type conversion functions
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_type_conversion_functions(global_env: &mut HashMap<String, Value>) {
    let conversion_functions = ["int", "float", "str", "bool"];

    for func_name in &conversion_functions {
        let builtin_name = format!("__builtin_{func_name}__");
        global_env.insert((*func_name).to_string(), Value::from_string(builtin_name));
    }
}

/// Add advanced utility functions
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_advanced_utility_functions(global_env: &mut HashMap<String, Value>) {
    let advanced_functions = [
        "reverse",
        "sort",
        "sum",
        "product",
        "unique",
        "flatten",
        "zip",
        "enumerate",
    ];

    for func_name in &advanced_functions {
        let builtin_name = format!("__builtin_{func_name}__");
        global_env.insert((*func_name).to_string(), Value::from_string(builtin_name));
    }
}

/// Add string utility functions
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_string_functions(global_env: &mut HashMap<String, Value>) {
    global_env.insert(
        "join".to_string(),
        Value::from_string("__builtin_join__".to_string()),
    );
    global_env.insert(
        "split".to_string(),
        Value::from_string("__builtin_split__".to_string()),
    );
}

/// Add random and time functions
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_random_time_functions(global_env: &mut HashMap<String, Value>) {
    global_env.insert(
        "random".to_string(),
        Value::from_string("__builtin_random__".to_string()),
    );
    global_env.insert(
        "random_int".to_string(),
        Value::from_string("__builtin_random_int__".to_string()),
    );
    global_env.insert(
        "timestamp".to_string(),
        Value::from_string("__builtin_timestamp__".to_string()),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_global_environment() {
        let env = init_global_environment();

        // Test constants
        assert_eq!(env.get("nil"), Some(&Value::Nil));

        // Test basic functions exist
        assert!(env.contains_key("format"));
        assert!(env.contains_key("HashMap"));
        assert!(env.contains_key("DataFrame"));

        // Test math functions
        assert!(env.contains_key("sqrt"));
        assert!(env.contains_key("sin"));
        assert!(env.contains_key("max"));

        // Test I/O functions
        assert!(env.contains_key("println"));
        assert!(env.contains_key("print"));

        // Test utilities
        assert!(env.contains_key("len"));
        assert!(env.contains_key("range"));

        // Test type conversions
        assert!(env.contains_key("int"));
        assert!(env.contains_key("str"));

        // Test advanced utilities
        assert!(env.contains_key("reverse"));
        assert!(env.contains_key("sort"));

        // Test string functions
        assert!(env.contains_key("join"));
        assert!(env.contains_key("split"));

        // Test random/time
        assert!(env.contains_key("random"));
        assert!(env.contains_key("timestamp"));
    }

    #[test]
    fn test_builtin_function_markers() {
        let env = init_global_environment();

        // Verify functions are stored as special string markers
        if let Some(Value::String(s)) = env.get("sqrt") {
            assert_eq!(&**s, "__builtin_sqrt__");
        } else {
            panic!("sqrt should be a string marker");
        }

        if let Some(Value::String(s)) = env.get("println") {
            assert_eq!(&**s, "__builtin_println__");
        } else {
            panic!("println should be a string marker");
        }
    }

    #[test]
    fn test_environment_count() {
        let env = init_global_environment();

        // Should have all builtin functions
        // 1 constant + 6 basic + 11 math + 3 I/O + 3 utility
        // + 4 conversion + 8 advanced + 2 string + 3 random/time = 41 total
        assert_eq!(env.len(), 41);
    }

    #[test]
    fn test_math_functions_complete() {
        let env = init_global_environment();

        let expected_math = [
            "sqrt", "pow", "abs", "min", "max", "floor", "ceil", "round", "sin", "cos", "tan",
        ];

        for func in &expected_math {
            assert!(env.contains_key(*func), "Missing math function: {func}");
        }
    }

    #[test]
    fn test_advanced_utilities_complete() {
        let env = init_global_environment();

        let expected_advanced = [
            "reverse",
            "sort",
            "sum",
            "product",
            "unique",
            "flatten",
            "zip",
            "enumerate",
        ];

        for func in &expected_advanced {
            assert!(env.contains_key(*func), "Missing advanced utility: {func}");
        }
    }
}
