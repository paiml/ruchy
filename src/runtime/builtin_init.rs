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
    add_environment_functions(&mut global_env);
    add_fs_functions(&mut global_env);
    add_stdlib003_functions(&mut global_env);  // STDLIB-003: User-friendly file I/O
    add_stdlib005_functions(&mut global_env);  // STDLIB-005: Directory walking
    add_path_functions(&mut global_env);
    add_json_functions(&mut global_env);
    add_http_functions(&mut global_env);

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
        "DataFrame::new".to_string(),
        Value::from_string("__builtin_dataframe_new__".to_string()),
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
        "DataFrame::from_csv_string".to_string(),
        Value::from_string("__builtin_dataframe_from_csv_string__".to_string()),
    );
    global_env.insert(
        "DataFrame::from_json".to_string(),
        Value::from_string("__builtin_dataframe_from_json__".to_string()),
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
    // STDLIB-002: Added log, log10, random
    let math_functions = [
        "sqrt", "pow", "abs", "min", "max", "floor", "ceil", "round",
        "sin", "cos", "tan", "log", "log10", "random",
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
    // BUG-037: Test assertions
    global_env.insert(
        "assert_eq".to_string(),
        Value::from_string("__builtin_assert_eq__".to_string()),
    );
    global_env.insert(
        "assert".to_string(),
        Value::from_string("__builtin_assert__".to_string()),
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
    global_env.insert(
        "get_time_ms".to_string(),
        Value::from_string("__builtin_timestamp__".to_string()),
    );
    global_env.insert(
        "sleep".to_string(),
        Value::from_string("__builtin_sleep__".to_string()),
    );
}

/// Add environment functions
///
/// # Complexity
/// Cyclomatic complexity: 1 (within Toyota Way limits)
fn add_environment_functions(global_env: &mut HashMap<String, Value>) {
    global_env.insert(
        "env_args".to_string(),
        Value::from_string("__builtin_env_args__".to_string()),
    );
    global_env.insert(
        "env_var".to_string(),
        Value::from_string("__builtin_env_var__".to_string()),
    );
    global_env.insert(
        "env_set_var".to_string(),
        Value::from_string("__builtin_env_set_var__".to_string()),
    );
    global_env.insert(
        "env_remove_var".to_string(),
        Value::from_string("__builtin_env_remove_var__".to_string()),
    );
    global_env.insert(
        "env_vars".to_string(),
        Value::from_string("__builtin_env_vars__".to_string()),
    );
    global_env.insert(
        "env_current_dir".to_string(),
        Value::from_string("__builtin_env_current_dir__".to_string()),
    );
    global_env.insert(
        "env_set_current_dir".to_string(),
        Value::from_string("__builtin_env_set_current_dir__".to_string()),
    );
    global_env.insert(
        "env_temp_dir".to_string(),
        Value::from_string("__builtin_env_temp_dir__".to_string()),
    );
}

/// Register file system functions in global environment
/// Phase 2: `STDLIB_ACCESS_PLAN` - File System Module (12 functions)
fn add_fs_functions(global_env: &mut HashMap<String, Value>) {
    global_env.insert(
        "fs_read".to_string(),
        Value::from_string("__builtin_fs_read__".to_string()),
    );
    global_env.insert(
        "fs_write".to_string(),
        Value::from_string("__builtin_fs_write__".to_string()),
    );
    global_env.insert(
        "fs_exists".to_string(),
        Value::from_string("__builtin_fs_exists__".to_string()),
    );
    global_env.insert(
        "fs_create_dir".to_string(),
        Value::from_string("__builtin_fs_create_dir__".to_string()),
    );
    global_env.insert(
        "fs_remove_file".to_string(),
        Value::from_string("__builtin_fs_remove_file__".to_string()),
    );
    global_env.insert(
        "fs_remove_dir".to_string(),
        Value::from_string("__builtin_fs_remove_dir__".to_string()),
    );
    global_env.insert(
        "fs_copy".to_string(),
        Value::from_string("__builtin_fs_copy__".to_string()),
    );
    global_env.insert(
        "fs_rename".to_string(),
        Value::from_string("__builtin_fs_rename__".to_string()),
    );
    global_env.insert(
        "fs_metadata".to_string(),
        Value::from_string("__builtin_fs_metadata__".to_string()),
    );
    global_env.insert(
        "fs_read_dir".to_string(),
        Value::from_string("__builtin_fs_read_dir__".to_string()),
    );
    global_env.insert(
        "fs_canonicalize".to_string(),
        Value::from_string("__builtin_fs_canonicalize__".to_string()),
    );
    global_env.insert(
        "fs_is_file".to_string(),
        Value::from_string("__builtin_fs_is_file__".to_string()),
    );
}

/// Register STDLIB-003: User-friendly file I/O aliases
/// Provides intuitive names for common file operations
/// Complexity: 1 (simple registration)
fn add_stdlib003_functions(global_env: &mut HashMap<String, Value>) {
    // STDLIB-003: Advanced File I/O Functions
    global_env.insert(
        "read_file".to_string(),
        Value::from_string("__builtin_read_file__".to_string()),
    );
    global_env.insert(
        "write_file".to_string(),
        Value::from_string("__builtin_write_file__".to_string()),
    );
    global_env.insert(
        "file_exists".to_string(),
        Value::from_string("__builtin_file_exists__".to_string()),
    );
    global_env.insert(
        "append_file".to_string(),
        Value::from_string("__builtin_append_file__".to_string()),
    );
    global_env.insert(
        "delete_file".to_string(),
        Value::from_string("__builtin_delete_file__".to_string()),
    );
}

/// Register STDLIB-005: Multi-Threaded Directory Walking + Text Search
/// Provides directory traversal and text search functionality
/// Complexity: 1 (simple registration)
fn add_stdlib005_functions(global_env: &mut HashMap<String, Value>) {
    // STDLIB-005: Directory Walking Functions
    global_env.insert(
        "walk".to_string(),
        Value::from_string("__builtin_walk__".to_string()),
    );
}

/// Register path functions in global environment
/// Phase 3: `STDLIB_ACCESS_PLAN` - Path Module (13 functions)
fn add_path_functions(global_env: &mut HashMap<String, Value>) {
    global_env.insert("path_join".to_string(), Value::from_string("__builtin_path_join__".to_string()));
    global_env.insert("path_join_many".to_string(), Value::from_string("__builtin_path_join_many__".to_string()));
    global_env.insert("path_parent".to_string(), Value::from_string("__builtin_path_parent__".to_string()));
    global_env.insert("path_file_name".to_string(), Value::from_string("__builtin_path_file_name__".to_string()));
    global_env.insert("path_file_stem".to_string(), Value::from_string("__builtin_path_file_stem__".to_string()));
    global_env.insert("path_extension".to_string(), Value::from_string("__builtin_path_extension__".to_string()));
    global_env.insert("path_is_absolute".to_string(), Value::from_string("__builtin_path_is_absolute__".to_string()));
    global_env.insert("path_is_relative".to_string(), Value::from_string("__builtin_path_is_relative__".to_string()));
    global_env.insert("path_canonicalize".to_string(), Value::from_string("__builtin_path_canonicalize__".to_string()));
    global_env.insert("path_with_extension".to_string(), Value::from_string("__builtin_path_with_extension__".to_string()));
    global_env.insert("path_with_file_name".to_string(), Value::from_string("__builtin_path_with_file_name__".to_string()));
    global_env.insert("path_components".to_string(), Value::from_string("__builtin_path_components__".to_string()));
    global_env.insert("path_normalize".to_string(), Value::from_string("__builtin_path_normalize__".to_string()));
}

/// Register JSON functions in global environment
/// Phase 4: `STDLIB_ACCESS_PLAN` - JSON Module (10 functions)
fn add_json_functions(global_env: &mut HashMap<String, Value>) {
    global_env.insert("json_parse".to_string(), Value::from_string("__builtin_json_parse__".to_string()));
    global_env.insert("json_stringify".to_string(), Value::from_string("__builtin_json_stringify__".to_string()));
    global_env.insert("json_pretty".to_string(), Value::from_string("__builtin_json_pretty__".to_string()));
    global_env.insert("json_read".to_string(), Value::from_string("__builtin_json_read__".to_string()));
    global_env.insert("json_write".to_string(), Value::from_string("__builtin_json_write__".to_string()));
    global_env.insert("json_validate".to_string(), Value::from_string("__builtin_json_validate__".to_string()));
    global_env.insert("json_type".to_string(), Value::from_string("__builtin_json_type__".to_string()));
    global_env.insert("json_merge".to_string(), Value::from_string("__builtin_json_merge__".to_string()));
    global_env.insert("json_get".to_string(), Value::from_string("__builtin_json_get__".to_string()));
    global_env.insert("json_set".to_string(), Value::from_string("__builtin_json_set__".to_string()));
}

/// Phase 5: STDLIB-PHASE-5 - HTTP Module (4 functions)
fn add_http_functions(global_env: &mut HashMap<String, Value>) {
    global_env.insert("http_get".to_string(), Value::from_string("__builtin_http_get__".to_string()));
    global_env.insert("http_post".to_string(), Value::from_string("__builtin_http_post__".to_string()));
    global_env.insert("http_put".to_string(), Value::from_string("__builtin_http_put__".to_string()));
    global_env.insert("http_delete".to_string(), Value::from_string("__builtin_http_delete__".to_string()));
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
        // 1 constant + 9 basic + 11 math + 3 I/O + 3 utility
        // + 4 conversion + 8 advanced + 2 string + 5 random/time + 8 env + 12 fs + 13 path + 10 json = 89 total
        // env functions: env_args, env_var, env_set_var, env_remove_var, env_vars,
        //                env_current_dir, env_set_current_dir, env_temp_dir
        // fs functions: fs_read, fs_write, fs_exists, fs_create_dir, fs_remove_file,
        //               fs_remove_dir, fs_copy, fs_rename, fs_metadata, fs_read_dir,
        //               fs_canonicalize, fs_is_file
        // path functions: path_join, path_join_many, path_parent, path_file_name, path_file_stem,
        //                 path_extension, path_is_absolute, path_is_relative, path_canonicalize,
        //                 path_with_extension, path_with_file_name, path_components, path_normalize
        // json functions: json_parse, json_stringify, json_pretty, json_read, json_write,
        //                 json_validate, json_type, json_merge, json_get, json_set
        // test functions: assert, assert_eq (added in v3.86.0 for BUG-037)
        // math functions: log, log10, random (added in STDLIB-002: 3 new)
        // file I/O functions: append_file, delete_file (STDLIB-003: 2 new, others existed)
        // string/array functions: substring, slice, join, unique, zip, enumerate (STDLIB-004: 6 new)
        // directory walking: walk (STDLIB-005: 1 new)
        // Total: 89 base + 3 STDLIB-002 + 2 STDLIB-003 + 6 STDLIB-004 + 1 STDLIB-005 + 2 misc = 103
        assert_eq!(env.len(), 103);
    }

    #[test]
    fn test_math_functions_complete() {
        let env = init_global_environment();

        let expected_math = [
            "sqrt", "pow", "abs", "min", "max", "floor", "ceil", "round", "sin", "cos", "tan",
            "log", "log10", "random",  // STDLIB-002: Advanced math functions
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
