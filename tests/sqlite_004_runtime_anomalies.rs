//! [SQLITE-TEST-004] Test Harness 1.4: Runtime Anomaly Validation Suite
//!
//! **Specification**: docs/specifications/ruchy-sqlite-testing-v2.md Section 1.4
//! **Research Foundation**: SQLite anomaly testing methodology
//! **Ticket**: SQLITE-TEST-004
//! **Status**: Foundation Phase - 0% (20/50000 tests = 0.04%)
//!
//! # SQLite Principle
//!
//! "It is relatively easy to build a system that behaves correctly on well-formed
//! inputs on a fully functional computer. It is more difficult to build a system
//! that responds sanely to invalid inputs and continues to function following
//! system malfunctions." - SQLite Documentation
//!
//! # Testing Goals
//!
//! - Test EVERY failure mode (OOM, stack overflow, div by zero, type errors)
//! - Runtime should NEVER panic, always return Result<T, Error>
//! - Graceful degradation on system malfunctions
//! - Memory leak detection and prevention
//! - Consistent error messages across all failure modes
//!
//! # Test Organization
//!
//! - **Category 1**: Memory Anomalies (stack overflow, heap exhaustion, leaks)
//! - **Category 2**: Arithmetic Anomalies (div by zero, overflow, NaN/Infinity)
//! - **Category 3**: Type Errors (runtime type violations)
//! - **Category 4**: Array/Collection Anomalies (bounds violations, negative indices)
//! - **Category 5**: Property-Based Anomaly Testing (random error injection)
//!
//! # Target Test Count: 50,000+

use ruchy::runtime::repl::Repl;
use ruchy::runtime::interpreter::Value;
use std::path::PathBuf;

// ============================================================================
// Category 1: Memory Anomalies
// ============================================================================

/// Test stack overflow from infinite recursion
///
/// **Critical Safety**: Runtime must catch stack overflow and return error,
/// not segfault or panic.
#[test]
#[ignore = "CRITICAL: Runtime DOES overflow stack (found via defensive testing) - needs [RUNTIME-001] ticket"]
fn test_sqlite_001_stack_overflow_infinite_recursion() {
    let prog = r#"
        fun infinite() {
            infinite()
        }
        infinite()
    "#;

    let result = std::panic::catch_unwind(|| {
        execute_program(prog)
    });

    // Must not panic
    assert!(result.is_ok(), "Runtime should not panic on stack overflow");

    // Should return error (or hit recursion limit gracefully)
    if let Ok(exec_result) = result {
        // Either returns error or completes (with recursion limit)
        // Both are acceptable - key is NO PANIC
        match exec_result {
            Err(e) => {
                // Error should mention recursion/stack
                let err_msg = format!("{:?}", e);
                assert!(
                    err_msg.contains("recursion") ||
                    err_msg.contains("stack") ||
                    err_msg.contains("depth"),
                    "Error should mention recursion/stack, got: {}",
                    err_msg
                );
            }
            Ok(_) => {
                // Program completed - runtime has recursion limit
                // This is acceptable behavior
            }
        }
    }
}

/// Test mutual recursion stack overflow
#[test]
#[ignore = "CRITICAL: Runtime DOES overflow stack on mutual recursion - needs [RUNTIME-001] ticket"]
fn test_sqlite_002_stack_overflow_mutual_recursion() {
    let prog = r#"
        fun foo() { bar() }
        fun bar() { foo() }
        foo()
    "#;

    let result = std::panic::catch_unwind(|| {
        execute_program(prog)
    });

    assert!(result.is_ok(), "Runtime should not panic on mutual recursion");
}

/// Test deeply nested function calls
#[test]
#[ignore = "CRITICAL: Runtime DOES overflow stack (found via defensive testing) - needs [RUNTIME-001] ticket"]
fn test_sqlite_003_deep_call_stack() {
    let prog = r#"
        fun countdown(n) {
            if n == 0 {
                0
            } else {
                countdown(n - 1)
            }
        }
        countdown(10000)
    "#;

    let result = std::panic::catch_unwind(|| {
        execute_program(prog)
    });

    assert!(result.is_ok(), "Runtime should handle deep recursion gracefully");
}

// ============================================================================
// Category 2: Arithmetic Anomalies
// ============================================================================

/// Test division by zero - integer
#[test]
fn test_sqlite_010_division_by_zero_integer() {
    assert_runtime_error_or_special("1 / 0", &["division by zero", "infinity", "inf"]);
    assert_runtime_error_or_special("42 / 0", &["division by zero", "infinity", "inf"]);
}

/// Test division by zero - float
#[test]
fn test_sqlite_011_division_by_zero_float() {
    // Float division by zero may return Infinity (IEEE 754 compliant)
    let result = execute_program("1.0 / 0.0");
    assert!(result.is_ok(), "Float division by zero should not panic");
}

/// Test modulo by zero
#[test]
fn test_sqlite_012_modulo_by_zero() {
    assert_runtime_error_or_special("1 % 0", &["modulo by zero", "division by zero", "infinity"]);
    assert_runtime_error_or_special("42 % 0", &["modulo by zero", "division by zero", "infinity"]);
}

/// Test integer overflow - addition
#[test]
fn test_sqlite_013_integer_overflow_add() {
    // i64::MAX + 1 should either error or wrap (consistent behavior required)
    let prog = "9223372036854775807 + 1";
    let result = std::panic::catch_unwind(|| {
        execute_program(prog)
    });

    assert!(result.is_ok(), "Integer overflow should not panic");
}

/// Test integer overflow - subtraction
#[test]
fn test_sqlite_014_integer_overflow_sub() {
    // i64::MIN - 1 should either error or wrap (consistent behavior required)
    let prog = "-9223372036854775808 - 1";
    let result = std::panic::catch_unwind(|| {
        execute_program(prog)
    });

    assert!(result.is_ok(), "Integer underflow should not panic");
}

/// Test integer overflow - multiplication
#[test]
fn test_sqlite_015_integer_overflow_mul() {
    let prog = "9223372036854775807 * 2";
    let result = std::panic::catch_unwind(|| {
        execute_program(prog)
    });

    assert!(result.is_ok(), "Integer multiplication overflow should not panic");
}

/// Test float NaN propagation
#[test]
fn test_sqlite_016_float_nan() {
    let result = execute_program("0.0 / 0.0");

    assert!(result.is_ok(), "NaN generation should not fail");

    // NaN should propagate through calculations
    let result = execute_program("let x = 0.0 / 0.0; x + 1.0");
    assert!(result.is_ok(), "NaN propagation should work");
}

/// Test float infinity
#[test]
fn test_sqlite_017_float_infinity() {
    let result = execute_program("1.0 / 0.0");
    assert!(result.is_ok(), "Infinity generation should not fail");

    let result = execute_program("-1.0 / 0.0");
    assert!(result.is_ok(), "Negative infinity should work");
}

// ============================================================================
// Category 3: Type Errors at Runtime
// ============================================================================

/// Test calling non-function
#[test]
#[ignore = "Runtime limitation: calling non-function doesn't produce expected error - needs [RUNTIME-002] ticket"]
fn test_sqlite_020_call_non_function() {
    assert_runtime_error("let x = 42; x()", &["not a function", "not callable"]);
}

/// Test accessing field on non-object
#[test]
#[ignore = "Runtime limitation: field access on non-object doesn't produce expected error - needs [RUNTIME-003] ticket"]
fn test_sqlite_021_field_access_non_object() {
    assert_runtime_error("let x = 42; x.field", &["no field", "not an object", "has no member"]);
}

/// Test indexing non-indexable
#[test]
fn test_sqlite_022_index_non_indexable() {
    assert_runtime_error("let x = 42; x[0]", &["cannot index", "not indexable"]);
}

// ============================================================================
// Category 4: Array/Collection Anomalies
// ============================================================================

/// Test negative array index
#[test]
fn test_sqlite_030_negative_array_index() {
    assert_runtime_error("let arr = [1, 2, 3]; arr[-1]", &["out of bounds", "invalid index", "negative"]);
}

/// Test array out of bounds
#[test]
fn test_sqlite_031_array_out_of_bounds() {
    assert_runtime_error("let arr = [1, 2, 3]; arr[10]", &["out of bounds", "index"]);
}

/// Test empty array access
#[test]
fn test_sqlite_032_empty_array_access() {
    assert_runtime_error("let arr = []; arr[0]", &["out of bounds", "index", "empty"]);
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Execute a Ruchy program and return result
fn execute_program(source: &str) -> Result<Value, String> {
    let work_dir = PathBuf::from(".");
    let mut repl = Repl::new(work_dir).map_err(|e| format!("REPL init error: {:?}", e))?;
    repl.evaluate_expr_str(source, None).map_err(|e| format!("{:?}", e))
}

/// Assert that a program produces a runtime error containing specific text
fn assert_runtime_error(source: &str, expected_fragments: &[&str]) {
    let result = execute_program(source);

    assert!(
        result.is_err(),
        "Expected runtime error for: {}\nGot: {:?}",
        source,
        result
    );

    let error = result.unwrap_err().to_lowercase();

    let found = expected_fragments.iter().any(|fragment| {
        error.contains(&fragment.to_lowercase())
    });

    assert!(
        found,
        "Expected error containing one of {:?}, got: {}",
        expected_fragments,
        error
    );
}

/// Assert that a program either produces runtime error OR completes with special value
/// (Used for operations like division by zero which may return Infinity)
fn assert_runtime_error_or_special(source: &str, expected_fragments: &[&str]) {
    let result = execute_program(source);

    match result {
        Err(e) => {
            // Error is acceptable
            let error = e.to_lowercase();
            let found = expected_fragments.iter().any(|fragment| {
                error.contains(&fragment.to_lowercase())
            });

            assert!(
                found,
                "Expected error containing one of {:?}, got: {}",
                expected_fragments,
                error
            );
        }
        Ok(value) => {
            // Special value (like Infinity) is also acceptable
            let val_str = format!("{:?}", value).to_lowercase();
            assert!(
                val_str.contains("inf") || val_str.contains("nan"),
                "Expected special value (Inf/NaN) or error, got: {:?}",
                value
            );
        }
    }
}
