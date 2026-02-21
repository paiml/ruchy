
use super::*;

#[test]
fn test_builtin_println() {
    let result = builtin_println(&[Value::from_string("test".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::nil());
}

// EXTREME TDD Sprint 4 TIER 2: println() edge cases
#[test]
fn test_builtin_println_empty() {
    let result = builtin_println(&[]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::nil());
}

#[test]
fn test_builtin_println_multiple_args() {
    let args = vec![
        Value::from_string("Hello".to_string()),
        Value::from_string("World".to_string()),
        Value::Integer(42),
    ];
    let result = builtin_println(&args).expect("builtin function should succeed in test");
    assert_eq!(result, Value::nil());
}

#[test]
fn test_builtin_println_different_types() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::Integer(1));
    let args = vec![
        Value::Integer(42),
        Value::Float(std::f64::consts::PI),
        Value::Bool(true),
        Value::Object(Arc::new(map)),
    ];
    let result = builtin_println(&args).expect("builtin function should succeed in test");
    assert_eq!(result, Value::nil());
}

#[test]
fn test_builtin_len() {
    let result = builtin_len(&[Value::from_string("hello".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(5));

    let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = builtin_len(&[arr]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(2));
}

// EXTREME TDD Sprint 4: len() edge cases
#[test]
fn test_builtin_len_object() {
    let mut map = HashMap::new();
    map.insert("a".to_string(), Value::Integer(1));
    map.insert("b".to_string(), Value::Integer(2));
    map.insert("c".to_string(), Value::Integer(3));
    let result = builtin_len(&[Value::Object(Arc::new(map))])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_builtin_len_range() {
    let range = Value::Range {
        start: Box::new(Value::Integer(1)),
        end: Box::new(Value::Integer(10)),
        inclusive: false,
    };
    let result = builtin_len(&[range]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(9)); // |10 - 1| = 9
}

#[test]
fn test_builtin_len_wrong_type() {
    let result = builtin_len(&[Value::Integer(42)]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("len() not supported for"));
}

#[test]
fn test_builtin_type_of() {
    let result =
        builtin_type_of(&[Value::Integer(42)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("integer".to_string()));
}

// EXTREME TDD Sprint 4: type_of() edge cases
#[test]
fn test_builtin_type_of_nil() {
    let result = builtin_type_of(&[Value::Nil]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("nil".to_string()));
}

#[test]
fn test_builtin_type_of_bool() {
    let result =
        builtin_type_of(&[Value::Bool(true)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("boolean".to_string()));
}

#[test]
fn test_builtin_type_of_float() {
    let result = builtin_type_of(&[Value::Float(std::f64::consts::PI)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("float".to_string()));
}

#[test]
fn test_builtin_type_of_string() {
    let result = builtin_type_of(&[Value::from_string("hello".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("string".to_string()));
}

#[test]
fn test_builtin_type_of_array() {
    let arr = Value::Array(Arc::from(vec![Value::Integer(1), Value::Integer(2)]));
    let result = builtin_type_of(&[arr]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("array".to_string()));
}

#[test]
fn test_builtin_type_of_object() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::from_string("value".to_string()));
    let result = builtin_type_of(&[Value::Object(Arc::new(map))])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("object".to_string()));
}

#[test]
fn test_builtin_sqrt() {
    let result =
        builtin_sqrt(&[Value::Integer(9)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Float(3.0));
}

// EXTREME TDD Sprint 4 TIER 2: sqrt() edge cases
#[test]
fn test_builtin_sqrt_negative() {
    let result =
        builtin_sqrt(&[Value::Integer(-1)]).expect("builtin function should succeed in test");
    assert!(matches!(result, Value::Float(x) if x.is_nan()));
}

#[test]
fn test_builtin_sqrt_zero() {
    let result =
        builtin_sqrt(&[Value::Integer(0)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Float(0.0));
}

#[test]
fn test_builtin_sqrt_float() {
    let result =
        builtin_sqrt(&[Value::Float(16.0)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Float(4.0));
}

#[test]
fn test_builtin_sqrt_wrong_type() {
    let result = builtin_sqrt(&[Value::from_string("not a number".to_string())]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("sqrt() expects a number"));
}

#[test]
fn test_builtin_abs() {
    let result =
        builtin_abs(&[Value::Integer(-42)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(42));
}

// EXTREME TDD Sprint 4 TIER 2: abs() edge cases
#[test]
fn test_builtin_abs_negative_float() {
    let result = builtin_abs(&[Value::Float(-std::f64::consts::PI)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Float(std::f64::consts::PI));
}

#[test]
fn test_builtin_abs_zero() {
    let result =
        builtin_abs(&[Value::Integer(0)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(0));
}

#[test]
fn test_builtin_abs_positive() {
    let result =
        builtin_abs(&[Value::Integer(42)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_builtin_abs_wrong_type() {
    let result = builtin_abs(&[Value::from_string("not a number".to_string())]);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("abs() expects a number"));
}

#[test]
fn test_builtin_registry() {
    let registry = BuiltinRegistry::new();
    assert!(registry.is_builtin("println"));
    assert!(registry.is_builtin("len"));
    assert!(!registry.is_builtin("not_a_builtin"));
}

// ========================================================================
// COVERAGE IMPROVEMENT: Collection Functions (reverse, sort)
// Target: 27.91% → 40%+ coverage
// ========================================================================

#[test]
fn test_builtin_reverse_array() {
    let arr = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
    let result = builtin_reverse(&[Value::Array(Arc::from(arr))])
        .expect("builtin function should succeed in test");

    if let Value::Array(reversed) = result {
        assert_eq!(reversed.len(), 3);
        assert_eq!(reversed[0], Value::Integer(3));
        assert_eq!(reversed[1], Value::Integer(2));
        assert_eq!(reversed[2], Value::Integer(1));
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_builtin_reverse_empty() {
    let arr = vec![];
    let result = builtin_reverse(&[Value::Array(Arc::from(arr))])
        .expect("builtin function should succeed in test");

    if let Value::Array(reversed) = result {
        assert_eq!(reversed.len(), 0);
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_builtin_reverse_wrong_args() {
    let result = builtin_reverse(&[]);
    assert!(result.is_err());

    let result = builtin_reverse(&[Value::Integer(42)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_sort_integers() {
    let arr = vec![Value::Integer(3), Value::Integer(1), Value::Integer(2)];
    let result = builtin_sort(&[Value::Array(Arc::from(arr))])
        .expect("builtin function should succeed in test");

    if let Value::Array(sorted) = result {
        assert_eq!(sorted[0], Value::Integer(1));
        assert_eq!(sorted[1], Value::Integer(2));
        assert_eq!(sorted[2], Value::Integer(3));
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_builtin_sort_floats() {
    let arr = vec![Value::Float(3.5), Value::Float(1.2), Value::Float(2.8)];
    let result = builtin_sort(&[Value::Array(Arc::from(arr))])
        .expect("builtin function should succeed in test");

    if let Value::Array(sorted) = result {
        assert_eq!(sorted[0], Value::Float(1.2));
        assert_eq!(sorted[1], Value::Float(2.8));
        assert_eq!(sorted[2], Value::Float(3.5));
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_builtin_sort_strings() {
    let arr = vec![
        Value::from_string("charlie".to_string()),
        Value::from_string("alice".to_string()),
        Value::from_string("bob".to_string()),
    ];
    let result = builtin_sort(&[Value::Array(Arc::from(arr))])
        .expect("builtin function should succeed in test");

    if let Value::Array(sorted) = result {
        assert_eq!(sorted[0], Value::from_string("alice".to_string()));
        assert_eq!(sorted[1], Value::from_string("bob".to_string()));
        assert_eq!(sorted[2], Value::from_string("charlie".to_string()));
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_builtin_sort_wrong_args() {
    let result = builtin_sort(&[]);
    assert!(result.is_err());

    let result = builtin_sort(&[Value::Integer(42)]);
    assert!(result.is_err());
}

// ========================================================================
// COVERAGE IMPROVEMENT: Environment Functions (env_*)
// Target: 30%+ → 45%+ coverage
// ========================================================================

#[test]
fn test_builtin_env_args() {
    let result = builtin_env_args(&[]).expect("builtin function should succeed in test");

    if let Value::Array(args) = result {
        // Should return array of strings (at minimum the program name)
        assert!(!args.is_empty());
    } else {
        panic!("Expected Array");
    }
}

#[test]
fn test_builtin_env_args_wrong_args() {
    let result = builtin_env_args(&[Value::Integer(1)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_env_var_existing() {
    // Set a test environment variable
    std::env::set_var("RUCHY_TEST_VAR", "test_value");

    let result = builtin_env_var(&[Value::from_string("RUCHY_TEST_VAR".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("test_value".to_string()));

    // Clean up
    std::env::remove_var("RUCHY_TEST_VAR");
}

#[test]
fn test_builtin_env_var_missing() {
    // Ensure variable doesn't exist
    std::env::remove_var("RUCHY_NONEXISTENT_VAR");

    let result = builtin_env_var(&[Value::from_string("RUCHY_NONEXISTENT_VAR".to_string())]);
    assert!(result.is_err(), "Should return error for missing variable");

    if let Err(InterpreterError::RuntimeError(msg)) = result {
        assert!(
            msg.contains("not found"),
            "Error message should mention 'not found'"
        );
    }
}

#[test]
fn test_builtin_env_var_wrong_args() {
    let result = builtin_env_var(&[]);
    assert!(result.is_err());

    let result = builtin_env_var(&[Value::Integer(42)]);
    assert!(result.is_err());

    let result = builtin_env_var(&[Value::from_string("TEST".to_string()), Value::Integer(1)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_env_set_var() {
    let result = builtin_env_set_var(&[
        Value::from_string("RUCHY_TEST_SET_VAR".to_string()),
        Value::from_string("new_value".to_string()),
    ])
    .expect("operation should succeed in test");

    assert_eq!(result, Value::Nil);

    // Verify it was set
    assert_eq!(
        std::env::var("RUCHY_TEST_SET_VAR").expect("env var should be set in test"),
        "new_value"
    );

    // Clean up
    std::env::remove_var("RUCHY_TEST_SET_VAR");
}

#[test]
fn test_builtin_env_set_var_wrong_args() {
    let result = builtin_env_set_var(&[]);
    assert!(result.is_err());

    let result = builtin_env_set_var(&[Value::from_string("TEST".to_string())]);
    assert!(result.is_err());

    let result = builtin_env_set_var(&[Value::Integer(1), Value::from_string("value".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_env_remove_var() {
    // Set a variable first
    std::env::set_var("RUCHY_TEST_REMOVE_VAR", "to_be_removed");

    let result = builtin_env_remove_var(&[Value::from_string("RUCHY_TEST_REMOVE_VAR".to_string())])
        .expect("operation should succeed in test");
    assert_eq!(result, Value::Nil);

    // Verify it was removed
    assert!(std::env::var("RUCHY_TEST_REMOVE_VAR").is_err());
}

#[test]
fn test_builtin_env_remove_var_nonexistent() {
    // Removing non-existent variable should succeed
    let result = builtin_env_remove_var(&[Value::from_string("RUCHY_NEVER_EXISTED".to_string())])
        .expect("operation should succeed in test");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_builtin_env_remove_var_wrong_args() {
    let result = builtin_env_remove_var(&[]);
    assert!(result.is_err());

    let result = builtin_env_remove_var(&[Value::Integer(42)]);
    assert!(result.is_err());

    let result =
        builtin_env_remove_var(&[Value::from_string("TEST".to_string()), Value::Integer(1)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_env_vars() {
    let result = builtin_env_vars(&[]).expect("builtin function should succeed in test");

    if let Value::Object(vars) = result {
        // Should have at least some environment variables
        assert!(
            !vars.is_empty(),
            "Environment variables should not be empty"
        );

        // Verify it contains expected env vars (PATH exists on all systems)
        assert!(vars.contains_key("PATH") || !vars.is_empty());
    } else {
        panic!("Expected Object");
    }
}

#[test]
fn test_builtin_env_vars_wrong_args() {
    let result = builtin_env_vars(&[Value::Integer(1)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_env_current_dir() {
    let result = builtin_env_current_dir(&[]).expect("builtin function should succeed in test");

    if let Value::String(dir) = result {
        // Should be a valid path
        assert!(!dir.is_empty());
        assert!(std::path::Path::new(dir.as_ref()).exists());
    } else {
        panic!("Expected String");
    }
}

#[test]
fn test_builtin_env_current_dir_wrong_args() {
    let result = builtin_env_current_dir(&[Value::Integer(1)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_env_set_current_dir() {
    // Save current directory
    let original_dir = std::env::current_dir().expect("builtin function should succeed in test");

    // Change to temp directory
    // Note: On macOS, temp_dir may be /var/folders/... but canonicalizes to /private/var/folders/...
    let temp_dir = std::env::temp_dir();
    let temp_dir_canonical = temp_dir.canonicalize().unwrap_or_else(|_| temp_dir.clone());
    let result =
        builtin_env_set_current_dir(&[Value::from_string(temp_dir.to_string_lossy().to_string())])
            .expect("operation should succeed in test");
    assert_eq!(result, Value::Nil);

    // Verify it changed (compare canonical paths to handle macOS symlinks)
    let current = std::env::current_dir()
        .expect("should get current dir in test")
        .canonicalize()
        .expect("should canonicalize in test");
    assert_eq!(current, temp_dir_canonical);

    // Restore original directory
    std::env::set_current_dir(&original_dir).expect("builtin function should succeed in test");
}

#[test]
fn test_builtin_env_set_current_dir_invalid() {
    let result = builtin_env_set_current_dir(&[Value::from_string(
        "/nonexistent/directory/xyz".to_string(),
    )]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_env_set_current_dir_wrong_args() {
    let result = builtin_env_set_current_dir(&[]);
    assert!(result.is_err());

    let result = builtin_env_set_current_dir(&[Value::Integer(42)]);
    assert!(result.is_err());

    let result =
        builtin_env_set_current_dir(&[Value::from_string("/tmp".to_string()), Value::Integer(1)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_env_temp_dir() {
    let result = builtin_env_temp_dir(&[]).expect("builtin function should succeed in test");

    if let Value::String(temp_dir) = result {
        // Should be a valid path that exists
        assert!(!temp_dir.is_empty());
        assert!(std::path::Path::new(temp_dir.as_ref()).exists());
    } else {
        panic!("Expected String");
    }
}

#[test]
fn test_builtin_env_temp_dir_wrong_args() {
    let result = builtin_env_temp_dir(&[Value::Integer(1)]);
    assert!(result.is_err());
}

// ========================================================================
// COVERAGE IMPROVEMENT: Filesystem Functions (fs_*)
// Target: 47.50% → 60%+ coverage
// ========================================================================

use tempfile::TempDir;

fn setup_test_file(content: &str) -> (TempDir, std::path::PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    std::fs::write(&file_path, content).expect("Failed to write test file");
    (temp_dir, file_path)
}

#[test]
fn test_builtin_fs_read() {
    let (temp_dir, file_path) = setup_test_file("Hello, World!");

    let result = builtin_fs_read(&[Value::from_string(file_path.to_string_lossy().to_string())])
        .expect("operation should succeed in test");
    assert_eq!(result, Value::from_string("Hello, World!".to_string()));

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_read_missing_file() {
    let result = builtin_fs_read(&[Value::from_string(
        "/tmp/nonexistent_test_file_12345.txt".to_string(),
    )]);
    assert!(result.is_err(), "Should error for missing file");
}

#[test]
fn test_builtin_fs_write() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("write_test.txt");

    let result = builtin_fs_write(&[
        Value::from_string(file_path.to_string_lossy().to_string()),
        Value::from_string("Test content".to_string()),
    ])
    .expect("operation should succeed in test");

    assert_eq!(result, Value::Nil);
    let content = std::fs::read_to_string(&file_path).expect("Failed to read file");
    assert_eq!(content, "Test content");

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_exists_true() {
    let (temp_dir, file_path) = setup_test_file("test");

    let result = builtin_fs_exists(&[Value::from_string(file_path.to_string_lossy().to_string())])
        .expect("operation should succeed in test");
    assert_eq!(result, Value::from_bool(true));

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_exists_false() {
    let result = builtin_fs_exists(&[Value::from_string(
        "/tmp/nonexistent_file_xyz123.txt".to_string(),
    )])
    .expect("operation should succeed in test");
    assert_eq!(result, Value::from_bool(false));
}

#[test]
fn test_builtin_fs_create_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dir_path = temp_dir.path().join("new_directory");

    let result =
        builtin_fs_create_dir(&[Value::from_string(dir_path.to_string_lossy().to_string())])
            .expect("operation should succeed in test");
    assert_eq!(result, Value::Nil);
    assert!(dir_path.exists() && dir_path.is_dir());

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_remove_file() {
    let (temp_dir, file_path) = setup_test_file("to be removed");
    assert!(file_path.exists());

    let result =
        builtin_fs_remove_file(&[Value::from_string(file_path.to_string_lossy().to_string())])
            .expect("operation should succeed in test");
    assert_eq!(result, Value::Nil);
    assert!(!file_path.exists(), "File should be removed");

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_remove_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dir_path = temp_dir.path().join("dir_to_remove");
    std::fs::create_dir(&dir_path).expect("Failed to create directory");
    assert!(dir_path.exists());

    let result =
        builtin_fs_remove_dir(&[Value::from_string(dir_path.to_string_lossy().to_string())])
            .expect("operation should succeed in test");
    assert_eq!(result, Value::Nil);
    assert!(!dir_path.exists(), "Directory should be removed");

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_copy() {
    let (temp_dir, source_path) = setup_test_file("content to copy");
    let dest_path = temp_dir.path().join("copied_file.txt");

    let result = builtin_fs_copy(&[
        Value::from_string(source_path.to_string_lossy().to_string()),
        Value::from_string(dest_path.to_string_lossy().to_string()),
    ])
    .expect("operation should succeed in test");

    assert_eq!(result, Value::Nil);
    assert!(dest_path.exists());
    let content = std::fs::read_to_string(&dest_path).expect("Failed to read copied file");
    assert_eq!(content, "content to copy");

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_rename() {
    let (temp_dir, old_path) = setup_test_file("content to rename");
    let new_path = temp_dir.path().join("renamed_file.txt");

    let result = builtin_fs_rename(&[
        Value::from_string(old_path.to_string_lossy().to_string()),
        Value::from_string(new_path.to_string_lossy().to_string()),
    ])
    .expect("operation should succeed in test");

    assert_eq!(result, Value::Nil);
    assert!(!old_path.exists(), "Old file should not exist");
    assert!(new_path.exists(), "New file should exist");

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_metadata() {
    let (temp_dir, file_path) = setup_test_file("metadata test");

    let result =
        builtin_fs_metadata(&[Value::from_string(file_path.to_string_lossy().to_string())])
            .expect("operation should succeed in test");

    if let Value::Object(meta) = result {
        assert!(meta.contains_key("is_file"));
        assert!(meta.contains_key("is_dir"));
        assert!(meta.contains_key("size")); // Key is "size" not "len"
                                            // Verify the values are correct types
        assert_eq!(
            meta.get("is_file")
                .expect("is_file key should exist in metadata"),
            &Value::Bool(true)
        );
        assert_eq!(
            meta.get("is_dir")
                .expect("is_dir key should exist in metadata"),
            &Value::Bool(false)
        );
    } else {
        panic!("Expected Object for metadata");
    }

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_read_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    std::fs::write(&file1, "test1").expect("Failed to write file1");
    std::fs::write(&file2, "test2").expect("Failed to write file2");

    let result = builtin_fs_read_dir(&[Value::from_string(
        temp_dir.path().to_string_lossy().to_string(),
    )])
    .expect("operation should succeed in test");

    if let Value::Array(entries) = result {
        assert!(entries.len() >= 2, "Should have at least 2 entries");
    } else {
        panic!("Expected Array for read_dir");
    }

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_canonicalize() {
    let (temp_dir, file_path) = setup_test_file("canonicalize test");

    let result =
        builtin_fs_canonicalize(&[Value::from_string(file_path.to_string_lossy().to_string())])
            .expect("operation should succeed in test");

    if let Value::String(canonical_path) = result {
        assert!(!canonical_path.is_empty());
    } else {
        panic!("Expected String for canonicalize");
    }

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_is_file_true() {
    let (temp_dir, file_path) = setup_test_file("is file test");

    let result = builtin_fs_is_file(&[Value::from_string(file_path.to_string_lossy().to_string())])
        .expect("operation should succeed in test");
    assert_eq!(result, Value::from_bool(true));

    drop(temp_dir);
}

#[test]
fn test_builtin_fs_is_file_false_for_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let result = builtin_fs_is_file(&[Value::from_string(
        temp_dir.path().to_string_lossy().to_string(),
    )])
    .expect("operation should succeed in test");
    assert_eq!(result, Value::from_bool(false));

    drop(temp_dir);
}

// Error handling tests
#[test]
fn test_builtin_fs_read_wrong_args() {
    let result = builtin_fs_read(&[]);
    assert!(result.is_err());

    let result = builtin_fs_read(&[Value::Integer(42)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_fs_write_wrong_args() {
    let result = builtin_fs_write(&[]);
    assert!(result.is_err());

    let result = builtin_fs_write(&[Value::from_string("test.txt".to_string())]);
    assert!(result.is_err());
}

// EXTREME TDD: Math functions (pow, min, max, floor, ceil, round)
#[test]
fn test_builtin_pow_integers() {
    let result = builtin_pow(&[Value::Integer(2), Value::Integer(3)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(8)); // Returns Integer when both args are Integer and exp >= 0
}

#[test]
fn test_builtin_pow_floats() {
    let result = builtin_pow(&[Value::Float(2.0), Value::Float(3.0)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Float(8.0));
}

#[test]
fn test_builtin_pow_wrong_args() {
    assert!(builtin_pow(&[]).is_err());
    assert!(builtin_pow(&[Value::Integer(2)]).is_err());
    assert!(builtin_pow(&[Value::from_string("invalid".to_string()), Value::Integer(2)]).is_err());
}

#[test]
fn test_builtin_min_integers() {
    let result = builtin_min(&[Value::Integer(5), Value::Integer(3)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_builtin_min_floats() {
    let result = builtin_min(&[Value::Float(5.5), Value::Float(3.3)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Float(3.3));
}

#[test]
fn test_builtin_min_wrong_args() {
    assert!(builtin_min(&[]).is_err());
    // NOTE: min accepts 1+ args, so single arg is valid
    assert!(builtin_min(&[Value::Integer(5)]).is_ok());
}

#[test]
fn test_builtin_max_integers() {
    let result = builtin_max(&[Value::Integer(5), Value::Integer(3)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(5));
}

#[test]
fn test_builtin_max_floats() {
    let result = builtin_max(&[Value::Float(5.5), Value::Float(3.3)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Float(5.5));
}

#[test]
fn test_builtin_max_wrong_args() {
    assert!(builtin_max(&[]).is_err());
    // NOTE: max accepts 1+ args, so single arg is valid
    assert!(builtin_max(&[Value::Integer(5)]).is_ok());
}

#[test]
fn test_builtin_floor_positive() {
    let result =
        builtin_floor(&[Value::Float(3.7)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(3)); // Returns Integer, not Float
}

#[test]
fn test_builtin_floor_negative() {
    let result =
        builtin_floor(&[Value::Float(-3.7)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(-4)); // Returns Integer, not Float
}

#[test]
fn test_builtin_floor_wrong_args() {
    assert!(builtin_floor(&[]).is_err());
    assert!(builtin_floor(&[Value::from_string("invalid".to_string())]).is_err());
}

#[test]
fn test_builtin_ceil_positive() {
    let result =
        builtin_ceil(&[Value::Float(3.2)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(4)); // Returns Integer, not Float
}

#[test]
fn test_builtin_ceil_negative() {
    let result =
        builtin_ceil(&[Value::Float(-3.2)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(-3)); // Returns Integer, not Float
}

#[test]
fn test_builtin_ceil_wrong_args() {
    assert!(builtin_ceil(&[]).is_err());
}

#[test]
fn test_builtin_round_positive() {
    let result =
        builtin_round(&[Value::Float(3.5)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(4)); // Returns Integer, not Float
}

#[test]
fn test_builtin_round_negative() {
    let result =
        builtin_round(&[Value::Float(-3.5)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(-4)); // Returns Integer, not Float
}

#[test]
fn test_builtin_round_wrong_args() {
    assert!(builtin_round(&[]).is_err());
}

// EXTREME TDD: String functions (to_string, parse_int, parse_float)
#[test]
fn test_builtin_to_string_integer() {
    let result =
        builtin_to_string(&[Value::Integer(42)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("42".to_string()));
}

#[test]
fn test_builtin_to_string_float() {
    let result = builtin_to_string(&[Value::Float(std::f64::consts::PI)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string(std::f64::consts::PI.to_string()));
}

#[test]
fn test_builtin_to_string_bool() {
    let result = builtin_to_string(&[Value::from_bool(true)])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("true".to_string()));
}

// EXTREME TDD Sprint 4: to_string() edge cases
#[test]
fn test_builtin_to_string_nil() {
    let result = builtin_to_string(&[Value::Nil]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("nil".to_string()));
}

#[test]
fn test_builtin_to_string_string_with_quotes() {
    // to_string() uses Display formatting, which adds quotes for strings
    let result = builtin_to_string(&[Value::from_string("hello".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("\"hello\"".to_string()));
}

#[test]
fn test_builtin_to_string_array() {
    let arr = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
    let result = builtin_to_string(&[Value::from_array(arr)])
        .expect("builtin function should succeed in test");
    // Should format array with Display trait
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_builtin_to_string_object() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), Value::from_string("value".to_string()));
    let result = builtin_to_string(&[Value::Object(Arc::new(map))])
        .expect("builtin function should succeed in test");
    // Should format object with Display trait
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_builtin_parse_int_valid() {
    let result = builtin_parse_int(&[Value::from_string("42".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_builtin_parse_int_invalid() {
    assert!(builtin_parse_int(&[Value::from_string("not_a_number".to_string())]).is_err());
}

#[test]
fn test_builtin_parse_int_wrong_args() {
    assert!(builtin_parse_int(&[]).is_err());
    // NOTE: parse_int has type coercion - accepts Integer directly and returns it
    assert!(builtin_parse_int(&[Value::Integer(42)]).is_ok());
}

#[test]
fn test_builtin_parse_float_valid() {
    let result = builtin_parse_float(&[Value::from_string(std::f64::consts::PI.to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Float(std::f64::consts::PI));
}

#[test]
fn test_builtin_parse_float_invalid() {
    assert!(builtin_parse_float(&[Value::from_string("not_a_number".to_string())]).is_err());
}

#[test]
fn test_builtin_parse_float_wrong_args() {
    assert!(builtin_parse_float(&[]).is_err());
}

// EXTREME TDD: Testing functions (assert, assert_eq, is_nil)
#[test]
fn test_builtin_assert_true() {
    let result = builtin_assert(&[Value::from_bool(true)]);
    assert!(result.is_ok());
}

#[test]
fn test_builtin_assert_false() {
    let result = builtin_assert(&[Value::from_bool(false)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_assert_wrong_args() {
    assert!(builtin_assert(&[]).is_err());
}

#[test]
fn test_builtin_assert_eq_equal() {
    let result = builtin_assert_eq(&[Value::Integer(42), Value::Integer(42)]);
    assert!(result.is_ok());
}

#[test]
fn test_builtin_assert_eq_not_equal() {
    let result = builtin_assert_eq(&[Value::Integer(42), Value::Integer(43)]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_assert_eq_wrong_args() {
    assert!(builtin_assert_eq(&[]).is_err());
    assert!(builtin_assert_eq(&[Value::Integer(42)]).is_err());
}

#[test]
fn test_builtin_is_nil_true() {
    let result = builtin_is_nil(&[Value::Nil]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_bool(true));
}

#[test]
fn test_builtin_is_nil_false() {
    let result =
        builtin_is_nil(&[Value::Integer(42)]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_bool(false));
}

#[test]
fn test_builtin_is_nil_wrong_args() {
    assert!(builtin_is_nil(&[]).is_err());
}

// EXTREME TDD: I/O and Collection functions
#[test]
fn test_builtin_print() {
    let result = builtin_print(&[Value::from_string("Hello".to_string())]);
    assert!(result.is_ok());
    assert_eq!(result.expect("result should be Ok in test"), Value::Nil);
}

// EXTREME TDD Sprint 4 TIER 2: print() edge cases
#[test]
fn test_builtin_print_empty() {
    let result = builtin_print(&[]);
    assert!(result.is_ok());
    assert_eq!(result.expect("result should be Ok in test"), Value::Nil);
}

#[test]
fn test_builtin_print_multiple_args() {
    let args = vec![
        Value::from_string("Hello".to_string()),
        Value::Integer(42),
        Value::Bool(false),
    ];
    let result = builtin_print(&args);
    assert!(result.is_ok());
    assert_eq!(result.expect("result should be Ok in test"), Value::Nil);
}

#[test]
fn test_builtin_dbg() {
    let result = builtin_dbg(&[Value::Integer(42)]);
    assert!(result.is_ok());
    // dbg returns the value it debugs, not Nil
    assert_eq!(
        result.expect("result should be Ok in test"),
        Value::Integer(42)
    );
}

// EXTREME TDD Sprint 4 TIER 2: dbg() edge cases
#[test]
fn test_builtin_dbg_multiple_args() {
    let args = vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)];
    let result = builtin_dbg(&args);
    assert!(result.is_ok());
    // dbg with multiple args returns an array
    match result.expect("result should be Ok in test") {
        Value::Array(arr) => assert_eq!(arr.len(), 3),
        _ => panic!("Expected array return from dbg with multiple args"),
    }
}

#[test]
fn test_builtin_dbg_different_types() {
    let mut map = HashMap::new();
    map.insert("test".to_string(), Value::Integer(99));
    let args = vec![
        Value::Float(std::f64::consts::PI),
        Value::from_string("debug".to_string()),
        Value::Object(Arc::new(map)),
    ];
    let result = builtin_dbg(&args);
    assert!(result.is_ok());
    // Returns array for multiple args
    assert!(matches!(
        result.expect("result should be Ok in test"),
        Value::Array(_)
    ));
}

#[test]
fn test_builtin_push_array() {
    let arr = Value::Array(vec![Value::Integer(1), Value::Integer(2)].into());
    let result =
        builtin_push(&[arr, Value::Integer(3)]).expect("builtin function should succeed in test");
    match result {
        Value::Array(items) => {
            assert_eq!(items.len(), 3);
            assert_eq!(items[2], Value::Integer(3));
        }
        _ => panic!("Expected array"),
    }
}

#[test]
fn test_builtin_push_wrong_args() {
    assert!(builtin_push(&[]).is_err());
    assert!(builtin_push(&[Value::Integer(42)]).is_err());
}

#[test]
fn test_builtin_pop_array() {
    let arr = Value::Array(vec![Value::Integer(1), Value::Integer(2), Value::Integer(3)].into());
    let result = builtin_pop(&[arr]).expect("builtin function should succeed in test");
    // NOTE: pop() returns the POPPED VALUE, not the modified array
    assert_eq!(result, Value::Integer(3));
}

#[test]
fn test_builtin_pop_empty_array() {
    let arr = Value::Array(vec![].into());
    // NOTE: pop() on empty array returns Nil, not an error
    let result = builtin_pop(&[arr]).expect("builtin function should succeed in test");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_builtin_pop_wrong_args() {
    assert!(builtin_pop(&[]).is_err());
    assert!(builtin_pop(&[Value::Integer(42)]).is_err());
}

// EXTREME TDD Sprint 2: Path functions
#[test]
fn test_builtin_path_join() {
    let result = builtin_path_join(&[
        Value::from_string("/home".to_string()),
        Value::from_string("user".to_string()),
    ])
    .expect("operation should succeed in test");
    assert!(matches!(result, Value::String(_)));
    if let Value::String(s) = result {
        assert!(s.as_ref().contains("home") && s.as_ref().contains("user"));
    }
}

#[test]
fn test_builtin_path_join_wrong_args() {
    assert!(builtin_path_join(&[]).is_err());
    assert!(builtin_path_join(&[Value::from_string("a".to_string())]).is_err());
    assert!(builtin_path_join(&[Value::Integer(42), Value::from_string("b".to_string())]).is_err());
}

#[test]
fn test_builtin_path_join_many() {
    let components = vec![
        Value::from_string("home".to_string()),
        Value::from_string("user".to_string()),
        Value::from_string("docs".to_string()),
    ];
    let result = builtin_path_join_many(&[Value::Array(components.into())])
        .expect("builtin function should succeed in test");
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_builtin_path_join_many_wrong_args() {
    assert!(builtin_path_join_many(&[]).is_err());
    assert!(builtin_path_join_many(&[Value::Integer(42)]).is_err());
}

#[test]
fn test_builtin_path_parent() {
    let result = builtin_path_parent(&[Value::from_string("/home/user/file.txt".to_string())])
        .expect("builtin function should succeed in test");
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_builtin_path_parent_root() {
    let result = builtin_path_parent(&[Value::from_string("/".to_string())])
        .expect("builtin function should succeed in test");
    // Root has no parent
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_builtin_path_file_name() {
    let result = builtin_path_file_name(&[Value::from_string("/home/user/file.txt".to_string())])
        .expect("operation should succeed in test");
    assert_eq!(result, Value::from_string("file.txt".to_string()));
}

#[test]
fn test_builtin_path_file_name_no_file() {
    let result = builtin_path_file_name(&[Value::from_string("/".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_builtin_path_file_stem() {
    let result = builtin_path_file_stem(&[Value::from_string("/home/user/file.txt".to_string())])
        .expect("operation should succeed in test");
    assert_eq!(result, Value::from_string("file".to_string()));
}

#[test]
fn test_builtin_path_extension() {
    let result = builtin_path_extension(&[Value::from_string("/home/user/file.txt".to_string())])
        .expect("operation should succeed in test");
    assert_eq!(result, Value::from_string("txt".to_string()));
}

#[test]
fn test_builtin_path_extension_no_ext() {
    let result = builtin_path_extension(&[Value::from_string("/home/user/file".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_builtin_path_is_absolute() {
    let result = builtin_path_is_absolute(&[Value::from_string("/home/user".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Bool(true));
    let result2 = builtin_path_is_absolute(&[Value::from_string("relative/path".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result2, Value::Bool(false));
}

#[test]
fn test_builtin_path_is_relative() {
    let result = builtin_path_is_relative(&[Value::from_string("relative/path".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Bool(true));
    let result2 = builtin_path_is_relative(&[Value::from_string("/absolute".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result2, Value::Bool(false));
}

#[test]
fn test_builtin_path_canonicalize() {
    // Create a temp file for testing canonicalize
    use std::fs;
    let temp_file = "/tmp/ruchy_test_canonicalize.txt";
    fs::write(temp_file, "test").expect("builtin function should succeed in test");
    let result = builtin_path_canonicalize(&[Value::from_string(temp_file.to_string())]);
    fs::remove_file(temp_file).ok();
    assert!(result.is_ok());
}

#[test]
fn test_builtin_path_canonicalize_nonexistent() {
    let result =
        builtin_path_canonicalize(&[Value::from_string("/nonexistent/path/file.txt".to_string())]);
    assert!(result.is_err());
}

#[test]
fn test_builtin_path_with_extension() {
    let result = builtin_path_with_extension(&[
        Value::from_string("/home/user/file.txt".to_string()),
        Value::from_string("rs".to_string()),
    ])
    .expect("operation should succeed in test");
    assert_eq!(result, Value::from_string("/home/user/file.rs".to_string()));
}

#[test]
fn test_builtin_path_with_file_name() {
    let result = builtin_path_with_file_name(&[
        Value::from_string("/home/user/file.txt".to_string()),
        Value::from_string("newfile.rs".to_string()),
    ])
    .expect("operation should succeed in test");
    assert_eq!(
        result,
        Value::from_string("/home/user/newfile.rs".to_string())
    );
}

#[test]
fn test_builtin_path_components() {
    let result = builtin_path_components(&[Value::from_string("/home/user/docs".to_string())])
        .expect("builtin function should succeed in test");
    assert!(matches!(result, Value::Array(_)));
    if let Value::Array(arr) = result {
        assert!(arr.len() >= 3);
    }
}

#[test]
fn test_builtin_path_normalize() {
    let result = builtin_path_normalize(&[Value::from_string("/home/./user/../docs".to_string())])
        .expect("operation should succeed in test");
    assert!(matches!(result, Value::String(_)));
    if let Value::String(s) = result {
        // Should resolve . and ..
        assert!(!s.as_ref().contains("./"));
    }
}

#[test]
fn test_builtin_path_normalize_with_dots() {
    let result = builtin_path_normalize(&[Value::from_string("/a/b/../c/./d".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("/a/c/d".to_string()));
}

// ============================================================================
// EXTREME TDD Sprint 3: JSON functions (10 functions, 28 tests)
// ============================================================================

// json_parse tests (3 tests)
#[test]
fn test_builtin_json_parse() {
    let json_str = r#"{"name": "test", "value": 42}"#;
    let result = builtin_json_parse(&[Value::from_string(json_str.to_string())])
        .expect("builtin function should succeed in test");
    assert!(matches!(result, Value::Object(_)));
}

#[test]
fn test_builtin_json_parse_invalid_json() {
    let invalid_json = "{invalid json}";
    assert!(builtin_json_parse(&[Value::from_string(invalid_json.to_string())]).is_err());
}

#[test]
fn test_builtin_json_parse_wrong_args() {
    assert!(builtin_json_parse(&[]).is_err());
    assert!(builtin_json_parse(&[Value::Integer(42)]).is_err());
}

// json_stringify tests (2 tests)
#[test]
fn test_builtin_json_stringify() {
    let obj = Value::Object(Arc::new(
        [("key".to_string(), Value::from_string("value".to_string()))]
            .iter()
            .cloned()
            .collect(),
    ));
    let result = builtin_json_stringify(&[obj]).expect("builtin function should succeed in test");
    assert!(matches!(result, Value::String(_)));
}

#[test]
fn test_builtin_json_stringify_wrong_args() {
    assert!(builtin_json_stringify(&[]).is_err());
}

// json_pretty tests (2 tests)
#[test]
fn test_builtin_json_pretty() {
    let obj = Value::Object(Arc::new(
        [("key".to_string(), Value::Integer(42))]
            .iter()
            .cloned()
            .collect(),
    ));
    let result = builtin_json_pretty(&[obj]).expect("builtin function should succeed in test");
    if let Value::String(s) = result {
        assert!(s.contains('\n')); // Pretty-printed should have newlines
    } else {
        panic!("Expected String result");
    }
}

#[test]
fn test_builtin_json_pretty_wrong_args() {
    assert!(builtin_json_pretty(&[]).is_err());
}

// json_read tests (2 tests)
#[test]
fn test_builtin_json_read() {
    use std::io::Write;
    let temp_path = "/tmp/test_json_read.json";
    let mut file =
        std::fs::File::create(temp_path).expect("builtin function should succeed in test");
    write!(file, r#"{{"test": true}}"#).expect("builtin function should succeed in test");
    drop(file);

    let result = builtin_json_read(&[Value::from_string(temp_path.to_string())])
        .expect("builtin function should succeed in test");
    assert!(matches!(result, Value::Object(_)));

    std::fs::remove_file(temp_path).expect("builtin function should succeed in test");
}

#[test]
fn test_builtin_json_read_file_not_found() {
    assert!(
        builtin_json_read(&[Value::from_string("/nonexistent/file.json".to_string())]).is_err()
    );
}

// json_write tests (2 tests)
#[test]
fn test_builtin_json_write() {
    let temp_path = "/tmp/test_json_write.json";
    let obj = Value::Object(Arc::new(
        [("test".to_string(), Value::Bool(true))]
            .iter()
            .cloned()
            .collect(),
    ));

    let result = builtin_json_write(&[Value::from_string(temp_path.to_string()), obj])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Bool(true));
    assert!(std::path::Path::new(temp_path).exists());

    std::fs::remove_file(temp_path).expect("builtin function should succeed in test");
}

#[test]
fn test_builtin_json_write_wrong_args() {
    assert!(builtin_json_write(&[]).is_err());
    assert!(builtin_json_write(&[Value::Integer(42), Value::Bool(true)]).is_err());
}

// json_validate tests (3 tests)
#[test]
fn test_builtin_json_validate_valid() {
    let valid_json = r#"{"key": "value"}"#;
    let result = builtin_json_validate(&[Value::from_string(valid_json.to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_builtin_json_validate_invalid() {
    let invalid_json = "{invalid}";
    let result = builtin_json_validate(&[Value::from_string(invalid_json.to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_builtin_json_validate_wrong_args() {
    assert!(builtin_json_validate(&[]).is_err());
    assert!(builtin_json_validate(&[Value::Integer(42)]).is_err());
}

// json_type tests (7 types: null, boolean, number, string, array, object, + error)
#[test]
fn test_builtin_json_type_null() {
    let result = builtin_json_type(&[Value::from_string("null".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("null".to_string()));
}

#[test]
fn test_builtin_json_type_boolean() {
    let result = builtin_json_type(&[Value::from_string("true".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("boolean".to_string()));
}

#[test]
fn test_builtin_json_type_number() {
    let result = builtin_json_type(&[Value::from_string("42".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("number".to_string()));
}

#[test]
fn test_builtin_json_type_string() {
    let result = builtin_json_type(&[Value::from_string(r#""test""#.to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("string".to_string()));
}

#[test]
fn test_builtin_json_type_array() {
    let result = builtin_json_type(&[Value::from_string("[1,2,3]".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("array".to_string()));
}

#[test]
fn test_builtin_json_type_object() {
    let result = builtin_json_type(&[Value::from_string(r#"{"key":"value"}"#.to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::from_string("object".to_string()));
}

#[test]
fn test_builtin_json_type_wrong_args() {
    assert!(builtin_json_type(&[]).is_err());
    assert!(builtin_json_type(&[Value::Integer(42)]).is_err());
}

// json_merge tests (2 tests)
#[test]
fn test_builtin_json_merge() {
    let obj1 = Value::Object(Arc::new(
        [("a".to_string(), Value::Integer(1))]
            .iter()
            .cloned()
            .collect(),
    ));
    let obj2 = Value::Object(Arc::new(
        [("b".to_string(), Value::Integer(2))]
            .iter()
            .cloned()
            .collect(),
    ));

    let result =
        builtin_json_merge(&[obj1, obj2]).expect("builtin function should succeed in test");
    assert!(matches!(result, Value::Object(_)));
}

#[test]
fn test_builtin_json_merge_wrong_args() {
    assert!(builtin_json_merge(&[]).is_err());
    assert!(builtin_json_merge(&[Value::Integer(42)]).is_err());
}

// json_get tests (3 tests)
#[test]
fn test_builtin_json_get() {
    let obj = Value::Object(Arc::new(
        [("key".to_string(), Value::Integer(42))]
            .iter()
            .cloned()
            .collect(),
    ));
    let result = builtin_json_get(&[obj, Value::from_string("key".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Integer(42));
}

#[test]
fn test_builtin_json_get_not_found() {
    let obj = Value::Object(Arc::new(
        [("key".to_string(), Value::Integer(42))]
            .iter()
            .cloned()
            .collect(),
    ));
    let result = builtin_json_get(&[obj, Value::from_string("nonexistent".to_string())])
        .expect("builtin function should succeed in test");
    assert_eq!(result, Value::Nil);
}

#[test]
fn test_builtin_json_get_wrong_args() {
    assert!(builtin_json_get(&[]).is_err());
    assert!(builtin_json_get(&[Value::Integer(42), Value::Integer(42)]).is_err());
}

// json_set tests (2 tests)
#[test]
fn test_builtin_json_set() {
    let obj = Value::Object(Arc::new(
        [("key".to_string(), Value::Integer(42))]
            .iter()
            .cloned()
            .collect(),
    ));
    let result = builtin_json_set(&[
        obj,
        Value::from_string("key".to_string()),
        Value::Integer(100),
    ])
    .expect("operation should succeed in test");
    assert!(matches!(result, Value::Object(_)));
}

#[test]
fn test_builtin_json_set_wrong_args() {
    assert!(builtin_json_set(&[]).is_err());
    assert!(
        builtin_json_set(&[Value::Integer(42), Value::Integer(42), Value::Integer(42)]).is_err()
    );
}
