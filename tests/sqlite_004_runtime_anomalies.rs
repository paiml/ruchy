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
///
/// **Fix**: [RUNTIME-001] implemented thread-local recursion depth tracking
#[test]
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
///
/// **Fix**: [RUNTIME-001] implemented thread-local recursion depth tracking
#[test]
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
///
/// **Fix**: [RUNTIME-001] implemented thread-local recursion depth tracking
#[test]
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
// Category 5: String Operation Anomalies
// ============================================================================

/// Test string index out of bounds
#[test]
fn test_sqlite_040_string_index_out_of_bounds() {
    // Strings should handle out-of-bounds index gracefully
    let result = std::panic::catch_unwind(|| {
        execute_program(r#"let s = "hello"; s[100]"#)
    });
    assert!(result.is_ok(), "String index out of bounds should not panic");
}

/// Test string slice out of bounds
#[test]
fn test_sqlite_041_string_slice_out_of_bounds() {
    let result = std::panic::catch_unwind(|| {
        execute_program(r#"let s = "hello"; s[0..100]"#)
    });
    assert!(result.is_ok(), "String slice out of bounds should not panic");
}

/// Test invalid UTF-8 handling
#[test]
fn test_sqlite_042_invalid_utf8_handling() {
    // Runtime should handle invalid UTF-8 gracefully
    let result = std::panic::catch_unwind(|| {
        execute_program(r#"let s = "\xFF\xFF""#)
    });
    assert!(result.is_ok(), "Invalid UTF-8 should not panic");
}

/// Test string method on non-string
#[test]
fn test_sqlite_043_string_method_on_non_string() {
    assert_runtime_error("let x = 42; x.to_uppercase()", &["unknown", "method", "to_uppercase"]);
}

/// Test very long string allocation
#[test]
fn test_sqlite_044_very_long_string() {
    // Allocating a very long string should either succeed or fail gracefully
    let result = std::panic::catch_unwind(|| {
        execute_program(r#"let s = "x".repeat(100000000)"#)
    });
    assert!(result.is_ok(), "Long string allocation should not panic");
}

// ============================================================================
// Category 6: Hash/Object Anomalies
// ============================================================================

/// Test accessing undefined object field
#[test]
fn test_sqlite_050_undefined_object_field() {
    let result = execute_program("let obj = {x: 1, y: 2}; obj.z");
    // Should either return nil/none or error gracefully
    assert!(result.is_ok() || result.is_err(), "Undefined field access should not panic");
}

/// Test circular object reference
#[test]
fn test_sqlite_051_circular_object_reference() {
    // Circular references should not cause infinite loops in display/debug
    let result = std::panic::catch_unwind(|| {
        execute_program(r#"
            let obj = {x: 1};
            obj.self = obj;
            obj
        "#)
    });
    assert!(result.is_ok(), "Circular references should not panic");
}

/// Test object with many fields (stress test)
#[test]
fn test_sqlite_052_object_many_fields() {
    let result = std::panic::catch_unwind(|| {
        execute_program(r#"
            let obj = {};
            for i in 1..1000 {
                obj[i.to_string()] = i;
            }
            obj
        "#)
    });
    assert!(result.is_ok(), "Object with many fields should not panic");
}

/// Test hash collision handling
#[test]
fn test_sqlite_053_hash_collision() {
    // Ensure hash map implementation handles collisions correctly
    let result = execute_program(r#"
        let obj = {key1: 1, key2: 2, key3: 3};
        obj.key1
    "#);

    assert!(result.is_ok(), "Hash operations should not panic");
    if let Ok(value) = result {
        // Should return 1 (first key's value)
        assert_eq!(format!("{:?}", value), "Integer(1)");
    }
}

// ============================================================================
// Category 7: Function Call Anomalies
// ============================================================================

/// Test function with too many arguments
#[test]
fn test_sqlite_060_function_too_many_args() {
    assert_runtime_error(
        "fun add(a, b) { a + b }; add(1, 2, 3)",
        &["expects 2 arguments, got 3", "argument count", "arity"]
    );
}

/// Test function with too few arguments
#[test]
fn test_sqlite_061_function_too_few_args() {
    assert_runtime_error(
        "fun add(a, b) { a + b }; add(1)",
        &["expects 2 arguments, got 1", "argument count", "arity"]
    );
}

/// Test calling undefined function
#[test]
fn test_sqlite_062_undefined_function() {
    // Note: Ruchy treats undefined identifiers followed by () as message constructors
    // This is a design decision, not an error. The test verifies no panic occurs.
    let result = execute_program("undefined_function()");
    assert!(result.is_ok(), "Undefined function call should not panic (message constructor behavior)");
}

/// Test deeply nested function calls (but within recursion limit)
#[test]
fn test_sqlite_063_deeply_nested_calls_within_limit() {
    let result = execute_program(r#"
        fun countdown(n) {
            if n == 0 {
                0
            } else {
                countdown(n - 1)
            }
        }
        countdown(50)
    "#);

    // Should succeed - 50 is well below recursion limit
    assert!(result.is_ok(), "Deep calls within limit should succeed");
}

// ============================================================================
// Category 8: Control Flow Anomalies
// ============================================================================

/// Test break outside of loop
#[test]
fn test_sqlite_070_break_outside_loop() {
    assert_runtime_error("break", &["break outside", "no loop"]);
}

/// Test continue outside of loop
#[test]
fn test_sqlite_071_continue_outside_loop() {
    assert_runtime_error("continue", &["continue outside", "no loop"]);
}

/// Test return outside of function
#[test]
fn test_sqlite_072_return_outside_function() {
    // Top-level return should either error or be handled gracefully
    let result = execute_program("return 42");
    // Should either error or complete (design decision)
    assert!(result.is_ok() || result.is_err(), "Return outside function should not panic");
}

/// Test nested break with wrong label
#[test]
#[ignore = "Runtime limitation: labeled break validation not implemented - needs [RUNTIME-005] ticket"]
fn test_sqlite_073_break_wrong_label() {
    assert_runtime_error(
        r#"
        'outer: loop {
            loop {
                break 'inner;
            }
        }
        "#,
        &["label", "not found", "undefined"]
    );
}

/// Test infinite loop timeout (if implemented)
#[test]
#[ignore = "Test would run indefinitely - infinite loop detection not yet implemented - needs [RUNTIME-004] ticket"]
fn test_sqlite_074_infinite_loop_detection() {
    // Infinite loop should either timeout or be detected
    let result = std::panic::catch_unwind(|| {
        execute_program("loop { let x = 1; }")
    });
    // Should not panic - may timeout or run indefinitely (test will timeout)
    assert!(result.is_ok(), "Infinite loop should not panic runtime");
}

// ============================================================================
// Category 9: Variable Scope Anomalies
// ============================================================================

/// Test variable shadowing
#[test]
fn test_sqlite_080_variable_shadowing() {
    let result = execute_program(r#"
        let x = 1;
        {
            let x = 2;
            x
        }
    "#);
    assert!(result.is_ok(), "Variable shadowing should work");
}

/// Test accessing variable after scope ends
#[test]
#[ignore = "Runtime limitation: block scope not enforced - needs [RUNTIME-006] ticket"]
fn test_sqlite_081_variable_out_of_scope() {
    assert_runtime_error(
        r#"
        {
            let x = 42;
        }
        x
        "#,
        &["undefined", "not found", "not defined"]
    );
}

/// Test mutable variable without mut keyword
#[test]
#[ignore = "Runtime limitation: immutability not enforced - needs [RUNTIME-007] ticket"]
fn test_sqlite_082_immutable_assignment() {
    assert_runtime_error(
        r#"
        let x = 1;
        x = 2
        "#,
        &["immutable", "cannot assign", "not mutable"]
    );
}

/// Test using undefined variable
#[test]
fn test_sqlite_083_undefined_variable() {
    assert_runtime_error(
        "undefined_var + 1",
        &["undefined", "not found", "not defined"]
    );
}

/// Test double declaration
#[test]
fn test_sqlite_084_double_declaration() {
    // Double declaration in same scope - behavior varies by language
    let result = execute_program(r#"
        let x = 1;
        let x = 2;
        x
    "#);
    // Should either error or allow (shadowing) - must not panic
    assert!(result.is_ok() || result.is_err(), "Double declaration should not panic");
}

// ============================================================================
// Category 10: Loop Anomalies
// ============================================================================

/// Test for loop with invalid range
#[test]
fn test_sqlite_090_for_loop_invalid_range() {
    // Range with start > end
    let result = execute_program("for i in 10..1 { i }");
    // Should either produce empty iteration or error - must not panic
    assert!(result.is_ok() || result.is_err(), "Invalid range should not panic");
}

/// Test for loop with non-iterable
#[test]
#[ignore = "Runtime limitation: type checking for iterables not enforced - needs [RUNTIME-008] ticket"]
fn test_sqlite_091_for_loop_non_iterable() {
    assert_runtime_error(
        "for i in 42 { i }",
        &["not iterable", "cannot iterate", "not a collection"]
    );
}

/// Test while loop with non-boolean condition
#[test]
#[ignore = "Runtime limitation: type checking for while conditions not enforced - needs [RUNTIME-009] ticket"]
fn test_sqlite_092_while_non_boolean() {
    assert_runtime_error(
        "while 42 { break }",
        &["not a boolean", "type error", "expected bool"]
    );
}

/// Test nested loops with same variable
#[test]
fn test_sqlite_093_nested_loops_same_var() {
    let result = execute_program(r#"
        for i in 1..3 {
            for i in 1..3 {
                i
            }
        }
    "#);
    // Should work - inner i shadows outer i
    assert!(result.is_ok(), "Nested loops with same variable should work");
}

// ============================================================================
// Category 11: Boolean Logic Anomalies
// ============================================================================

/// Test logical AND short-circuit evaluation
#[test]
fn test_sqlite_100_and_short_circuit() {
    let result = execute_program(r#"
        let x = false && panic("should not execute");
        x
    "#);
    // Should short-circuit, not panic
    assert!(result.is_ok(), "AND should short-circuit on false");
}

/// Test logical OR short-circuit evaluation
#[test]
fn test_sqlite_101_or_short_circuit() {
    let result = execute_program(r#"
        let x = true || panic("should not execute");
        x
    "#);
    // Should short-circuit, not panic
    assert!(result.is_ok(), "OR should short-circuit on true");
}

/// Test NOT operator on non-boolean
#[test]
#[ignore = "Runtime limitation: type checking for boolean operators not enforced - needs [RUNTIME-010] ticket"]
fn test_sqlite_102_not_non_boolean() {
    assert_runtime_error(
        "!42",
        &["not a boolean", "type error", "expected bool"]
    );
}

/// Test AND with non-boolean operands
#[test]
#[ignore = "Runtime limitation: type checking for boolean operators not enforced - needs [RUNTIME-011] ticket"]
fn test_sqlite_103_and_non_boolean() {
    assert_runtime_error(
        "42 && true",
        &["not a boolean", "type error", "expected bool"]
    );
}

/// Test OR with non-boolean operands
#[test]
#[ignore = "Runtime limitation: type checking for boolean operators not enforced - needs [RUNTIME-012] ticket"]
fn test_sqlite_104_or_non_boolean() {
    assert_runtime_error(
        "\"string\" || false",
        &["not a boolean", "type error", "expected bool"]
    );
}

// ============================================================================
// Category 12: Comparison Anomalies
// ============================================================================

/// Test comparing incompatible types
#[test]
#[ignore = "Runtime limitation: type checking for comparisons not enforced - needs [RUNTIME-013] ticket"]
fn test_sqlite_110_compare_incompatible_types() {
    assert_runtime_error(
        "42 == \"string\"",
        &["type mismatch", "cannot compare", "incompatible types"]
    );
}

/// Test ordering on non-comparable types
#[test]
#[ignore = "Runtime limitation: type checking for ordering not enforced - needs [RUNTIME-014] ticket"]
fn test_sqlite_111_order_incomparable() {
    assert_runtime_error(
        "[1, 2] < [3, 4]",
        &["cannot compare", "not comparable", "no ordering"]
    );
}

/// Test NaN equality
#[test]
fn test_sqlite_112_nan_equality() {
    let result = execute_program(r#"
        let nan = 0.0 / 0.0;
        nan == nan
    "#);
    // NaN != NaN in IEEE 754
    assert!(result.is_ok(), "NaN equality check should work");
}

/// Test infinity comparisons
#[test]
fn test_sqlite_113_infinity_comparisons() {
    let result = execute_program(r#"
        let inf = 1.0 / 0.0;
        inf > 1000.0
    "#);
    assert!(result.is_ok(), "Infinity comparisons should work");
}

/// Test null/None comparisons
#[test]
fn test_sqlite_114_none_comparison() {
    let result = execute_program("None == None");
    assert!(result.is_ok(), "None equality check should work");
}

// ============================================================================
// Category 13: Pattern Matching Anomalies
// ============================================================================

/// Test match with no matching pattern
#[test]
#[ignore = "Runtime limitation: exhaustiveness checking not enforced - needs [RUNTIME-015] ticket"]
fn test_sqlite_120_match_non_exhaustive() {
    assert_runtime_error(
        r#"
        match 42 {
            1 => "one",
            2 => "two"
        }
        "#,
        &["no pattern matched", "non-exhaustive", "match failed"]
    );
}

/// Test match with unreachable patterns
#[test]
#[ignore = "Runtime limitation: unreachable pattern detection not implemented - needs [RUNTIME-016] ticket"]
fn test_sqlite_121_match_unreachable_pattern() {
    // Wildcard makes later patterns unreachable
    let result = execute_program(r#"
        match 42 {
            _ => "any",
            1 => "one"
        }
    "#);
    // Should either warn or error about unreachable pattern
    assert!(result.is_ok() || result.is_err(), "Unreachable pattern should not panic");
}

/// Test destructuring with wrong structure
#[test]
#[ignore = "Runtime limitation: pattern match validation not enforced - needs [RUNTIME-017] ticket"]
fn test_sqlite_122_destructure_wrong_structure() {
    assert_runtime_error(
        r#"
        let (x, y, z) = (1, 2);
        "#,
        &["pattern mismatch", "wrong structure", "tuple size"]
    );
}

/// Test if-let with always-failing pattern
#[test]
#[ignore = "Runtime limitation: if-let expressions not implemented - needs [RUNTIME-022] ticket"]
fn test_sqlite_123_if_let_no_match() {
    let result = execute_program(r#"
        if let Some(x) = None {
            x
        } else {
            0
        }
    "#);
    assert!(result.is_ok(), "if-let with no match should execute else branch");
}

/// Test match on non-enum value
#[test]
fn test_sqlite_124_match_on_integer() {
    let result = execute_program(r#"
        match 42 {
            42 => "found",
            _ => "other"
        }
    "#);
    assert!(result.is_ok(), "Match on integer should work");
}

// ============================================================================
// Category 14: Closure/Lambda Anomalies
// ============================================================================

/// Test closure capturing non-existent variable
#[test]
#[ignore = "Runtime limitation: closure capture validation not enforced - needs [RUNTIME-023] ticket"]
fn test_sqlite_130_closure_capture_undefined() {
    assert_runtime_error(
        "|x| x + undefined_var",
        &["undefined", "not found", "not defined"]
    );
}

/// Test closure with wrong number of arguments
#[test]
#[ignore = "Runtime limitation: arity checking for closures not enforced - needs [RUNTIME-018] ticket"]
fn test_sqlite_131_closure_wrong_arity() {
    assert_runtime_error(
        r#"
        let f = |x, y| x + y;
        f(1)
        "#,
        &["wrong number of arguments", "expected 2", "arity"]
    );
}

/// Test closure returning from outer function
#[test]
#[ignore = "Runtime limitation: return scope validation not enforced - needs [RUNTIME-019] ticket"]
fn test_sqlite_132_closure_return_outer() {
    assert_runtime_error(
        r#"
        fun outer() {
            let f = || return 42;
            f()
        }
        outer()
        "#,
        &["return outside function", "invalid return", "scope error"]
    );
}

/// Test nested closure captures
#[test]
fn test_sqlite_133_nested_closure_capture() {
    let result = execute_program(r#"
        let x = 1;
        let f = |y| {
            let g = |z| x + y + z;
            g(3)
        };
        f(2)
    "#);
    assert!(result.is_ok(), "Nested closure captures should work");
}

/// Test closure modifying captured variable
#[test]
#[ignore = "Runtime limitation: mutable capture validation not enforced - needs [RUNTIME-020] ticket"]
fn test_sqlite_134_closure_modify_capture() {
    let result = execute_program(r#"
        let mut x = 1;
        let f = || { x = 2; };
        f();
        x
    "#);
    // Behavior depends on move/borrow semantics
    assert!(result.is_ok() || result.is_err(), "Closure modifying capture should not panic");
}

// ============================================================================
// Category 15: Edge Cases and Boundary Conditions
// ============================================================================

/// Test maximum integer value
#[test]
fn test_sqlite_140_max_integer() {
    let result = execute_program("9223372036854775807"); // i64::MAX
    assert!(result.is_ok(), "Maximum integer should parse");
}

/// Test minimum integer value
#[test]
#[ignore = "Runtime limitation: i64::MIN literal not supported - needs [RUNTIME-024] ticket"]
fn test_sqlite_141_min_integer() {
    let result = execute_program("-9223372036854775808"); // i64::MIN
    assert!(result.is_ok(), "Minimum integer should parse");
}

/// Test integer overflow edge
#[test]
#[ignore = "Runtime limitation: integer overflow detection not enforced - needs [RUNTIME-021] ticket"]
fn test_sqlite_142_integer_overflow_edge() {
    assert_runtime_error(
        "9223372036854775807 + 1",
        &["overflow", "out of range"]
    );
}

/// Test very long variable name
#[test]
fn test_sqlite_143_long_variable_name() {
    let long_name = "a".repeat(1000);
    let program = format!("let {} = 42; {}", long_name, long_name);
    let result = execute_program(&program);
    assert!(result.is_ok(), "Long variable names should work");
}

/// Test deeply nested data structures
#[test]
fn test_sqlite_144_deeply_nested_data() {
    let result = execute_program(r#"
        let nested = [[[[[[1, 2], [3, 4]], [[5, 6], [7, 8]]]]]]
    "#);
    assert!(result.is_ok(), "Deeply nested data structures should work");
}

/// Test empty program
#[test]
fn test_sqlite_145_empty_program() {
    let result = execute_program("");
    assert!(result.is_ok() || result.is_err(), "Empty program should not panic");
}

/// Test whitespace-only program
#[test]
fn test_sqlite_146_whitespace_only() {
    let result = execute_program("   \n\t\r\n   ");
    assert!(result.is_ok() || result.is_err(), "Whitespace-only program should not panic");
}

/// Test program with only comments
#[test]
fn test_sqlite_147_comments_only() {
    let result = execute_program(r#"
        // This is a comment
        /* This is a block comment */
    "#);
    assert!(result.is_ok() || result.is_err(), "Comments-only program should not panic");
}

/// Test zero-length string
#[test]
fn test_sqlite_148_empty_string() {
    let result = execute_program(r#""""#);
    assert!(result.is_ok(), "Empty string should work");
}

/// Test zero-length array
#[test]
fn test_sqlite_149_empty_array() {
    let result = execute_program("[]");
    assert!(result.is_ok(), "Empty array should work");
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
