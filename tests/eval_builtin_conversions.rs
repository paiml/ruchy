//! EXTREME TDD: `eval_builtin.rs` Conversion & Time Functions
//!
//! Coverage Target: conversion + time functions in `eval_builtin.rs`
//! Functions: `str()`, `int()`, `float()`, `bool()`, `sleep()`, `timestamp()`

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    Command::cargo_bin("ruchy").expect("Failed to find ruchy binary")
}

// ============================================================================
// Conversion Functions: str() (__builtin_str__)
// ============================================================================

#[test]
fn test_str_from_integer() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(str(42))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_str_from_float() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(str(3.14))")
        .assert()
        .success()
        .stdout(predicate::str::contains("3.14"));
}

#[test]
fn test_str_from_boolean_true() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(str(true))")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_str_from_boolean_false() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(str(false))")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn test_str_from_string() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(str(\"hello\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("hello"));
}

// ============================================================================
// Conversion Functions: int() (__builtin_int__)
// ============================================================================

#[test]
fn test_int_from_string_positive() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(int(\"123\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("123"));
}

#[test]
fn test_int_from_string_negative() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(int(\"-456\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("-456"));
}

#[test]
fn test_int_from_float_truncate() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(int(3.7))")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_int_from_float_negative() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(int(-3.7))")
        .assert()
        .success()
        .stdout(predicate::str::contains("-3"));
}

#[test]
fn test_int_from_boolean_true() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(int(true))")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_int_from_boolean_false() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(int(false))")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_int_from_integer() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(int(42))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

// ============================================================================
// Conversion Functions: float() (__builtin_float__)
// ============================================================================

#[test]
fn test_float_from_string() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(float(\"3.14\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("3.14"));
}

#[test]
fn test_float_from_string_integer() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(float(\"42\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_float_from_integer() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(float(42))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn test_float_from_float() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(float(3.14))")
        .assert()
        .success()
        .stdout(predicate::str::contains("3.14"));
}

#[test]
fn test_float_from_boolean_true() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(float(true))")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn test_float_from_boolean_false() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(float(false))")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

// ============================================================================
// Conversion Functions: bool() (__builtin_bool__)
// ============================================================================

#[test]
fn test_bool_from_integer_nonzero() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(bool(42))")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_bool_from_integer_zero() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(bool(0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn test_bool_from_float_nonzero() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(bool(3.14))")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_bool_from_float_zero() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(bool(0.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn test_bool_from_string_nonempty() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(bool(\"hello\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_bool_from_string_empty() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(bool(\"\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn test_bool_from_boolean() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(bool(true), bool(false))")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"))
        .stdout(predicate::str::contains("false"));
}

// ============================================================================
// Time Functions: sleep() and timestamp()
// ============================================================================

#[test]
fn test_sleep_milliseconds() {
    ruchy_cmd()
        .arg("-e")
        .arg("sleep(10); println(\"awake\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("awake"));
}

#[test]
fn test_sleep_zero() {
    ruchy_cmd()
        .arg("-e")
        .arg("sleep(0); println(\"immediate\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("immediate"));
}

#[test]
fn test_timestamp_returns_number() {
    // timestamp() returns a number (milliseconds since epoch)
    ruchy_cmd()
        .arg("-e")
        .arg("let t = timestamp(); assert(t > 0)")
        .assert()
        .success();
}

#[test]
fn test_timestamp_increases() {
    // Two consecutive timestamps should have second >= first
    ruchy_cmd()
        .arg("-e")
        .arg("let t1 = timestamp(); sleep(1); let t2 = timestamp(); assert(t2 >= t1)")
        .assert()
        .success();
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_str_int_roundtrip() {
    // Property: int(str(n)) == n for integers
    for n in [-100, 0, 42, 999] {
        ruchy_cmd()
            .arg("-e")
            .arg(format!(
                "let s = str({n}); let back = int(s); assert_eq(back, {n})"
            ))
            .assert()
            .success();
    }
}

#[test]
fn property_str_float_roundtrip() {
    // Property: float(str(f)) ≈ f for floats
    for f in [0.0, 3.14, -2.5] {
        ruchy_cmd().arg("-e")
            .arg(format!("let s = str({f}); let back = float(s); assert(back >= {f} - 0.01); assert(back <= {f} + 0.01)"))
            .assert().success();
    }
}

#[test]
fn property_bool_truthy_values() {
    // Property: Non-zero/non-empty values are truthy
    let truthy = vec!["1", "42", "-1", "3.14", "\"x\""];
    for val in truthy {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("assert(bool({val}))"))
            .assert()
            .success();
    }
}

#[test]
fn property_bool_falsy_values() {
    // Property: Zero/empty values are falsy
    let falsy = vec!["0", "0.0", "\"\""];
    for val in falsy {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("assert(bool({val}) == false)"))
            .assert()
            .success();
    }
}

#[test]
fn property_int_truncates_towards_zero() {
    // Property: int() truncates towards zero
    let pairs = [(3.9, 3), (-3.9, -3), (2.1, 2), (-2.1, -2)];
    for (input, expected) in pairs {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("assert_eq(int({input}), {expected})"))
            .assert()
            .success();
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn integration_conversion_chain() {
    // str → int → float → bool → str
    ruchy_cmd()
        .arg("-e")
        .arg("let s = \"42\"; let i = int(s); let f = float(i); let b = bool(f); println(str(b))")
        .assert()
        .success()
        .stdout(predicate::str::contains("true"));
}

#[test]
fn integration_time_measurement() {
    // Measure duration using timestamps
    ruchy_cmd().arg("-e")
        .arg("let start = timestamp(); sleep(10); let end = timestamp(); let duration = end - start; assert(duration >= 10)")
        .assert().success();
}

#[test]
fn integration_type_coercion() {
    // Test implicit type conversions in expressions
    ruchy_cmd()
        .arg("-e")
        .arg("let x = int(\"5\"); let y = float(x); let z = x + y; println(z)")
        .assert()
        .success()
        .stdout(predicate::str::contains("10"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_int_max_string_length() {
    // Large integer string
    ruchy_cmd()
        .arg("-e")
        .arg("println(int(\"999999999\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("999999999"));
}

#[test]
fn edge_case_float_scientific_notation() {
    // Scientific notation (if supported)
    ruchy_cmd()
        .arg("-e")
        .arg("println(float(\"1.5e2\"))")
        .assert()
        .success();
}

#[test]
fn edge_case_bool_negative_zero() {
    // -0.0 should be falsy
    ruchy_cmd()
        .arg("-e")
        .arg("println(bool(-0.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("false"));
}

#[test]
fn edge_case_timestamp_consistency() {
    // Multiple timestamp calls in same expression
    ruchy_cmd()
        .arg("-e")
        .arg("let t1 = timestamp(); let t2 = timestamp(); assert(t2 >= t1)")
        .assert()
        .success();
}

// ============================================================================
// Error Handling
// ============================================================================

#[test]
fn error_int_invalid_string() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = int(\"not_a_number\")")
        .assert()
        .failure();
}

#[test]
fn error_float_invalid_string() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = float(\"invalid\")")
        .assert()
        .failure();
}

#[test]
fn error_sleep_negative() {
    // sleep() with negative duration should fail
    ruchy_cmd().arg("-e").arg("sleep(-100)").assert().failure();
}
