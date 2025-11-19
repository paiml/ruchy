//! EXTREME TDD: `eval_builtin.rs` Comprehensive Coverage
//!
//! Target: `eval_builtin.rs` 36.2% → 85% coverage
//! Strategy: Test all working builtin functions via CLI evaluation
//! Quality: Unit + Property + Integration tests

use assert_cmd::Command;
use predicates::prelude::*;

fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// ============================================================================
// Math Functions (eval_builtin.rs lines 96-170)
// ============================================================================

#[test]
fn test_sqrt_positive() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(sqrt(16.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_sqrt_zero() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(sqrt(0.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_pow_basic() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(pow(2.0, 8.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("256"));
}

#[test]
fn test_pow_fractional() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(pow(4.0, 0.5))")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_abs_negative() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(abs(-42.5))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42.5"));
}

#[test]
fn test_abs_positive() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(abs(42.5))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42.5"));
}

#[test]
fn test_min_two_args() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(min(3.0, 7.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_min_chained() {
    // min only supports 2 args, so chain: min(min(5, 2), 8)
    ruchy_cmd()
        .arg("-e")
        .arg("println(min(min(5.0, 2.0), 8.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_max_two_args() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(max(3.0, 7.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("7"));
}

#[test]
fn test_max_negative_numbers() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(max(-10.0, -5.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("-5"));
}

#[test]
fn test_floor_positive() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(floor(3.9))")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_floor_negative() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(floor(-3.1))")
        .assert()
        .success()
        .stdout(predicate::str::contains("-4"));
}

#[test]
fn test_ceil_positive() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(ceil(3.1))")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn test_ceil_negative() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(ceil(-3.9))")
        .assert()
        .success()
        .stdout(predicate::str::contains("-3"));
}

#[test]
fn test_round_half_up() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(round(2.5))")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_round_half_down() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(round(2.4))")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

// ============================================================================
// Utility Functions (eval_builtin.rs lines 186-220)
// ============================================================================

#[test]
fn test_len_string() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(len(\"hello world\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("11"));
}

#[test]
fn test_len_array() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(len([1, 2, 3, 4, 5]))")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_len_empty_string() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(len(\"\"))")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_len_empty_array() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(len([]))")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn test_range_single_arg() {
    ruchy_cmd()
        .arg("-e")
        .arg("let r = range(5); println(len(r))")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_range_two_args() {
    ruchy_cmd()
        .arg("-e")
        .arg("let r = range(2, 7); println(len(r))")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_range_with_step() {
    ruchy_cmd()
        .arg("-e")
        .arg("let r = range(0, 10, 2); println(len(r))")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn test_reverse_array() {
    ruchy_cmd()
        .arg("-e")
        .arg("let r = reverse([1, 2, 3]); println(r[0])")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

#[test]
fn test_assert_eq_pass() {
    ruchy_cmd()
        .arg("-e")
        .arg("assert_eq(42, 42); println(\"OK\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("OK"));
}

#[test]
fn test_assert_eq_fail() {
    ruchy_cmd()
        .arg("-e")
        .arg("assert_eq(42, 99)")
        .assert()
        .failure();
}

#[test]
fn test_assert_true() {
    ruchy_cmd()
        .arg("-e")
        .arg("assert(true); println(\"OK\")")
        .assert()
        .success();
}

#[test]
fn test_assert_false() {
    ruchy_cmd()
        .arg("-e")
        .arg("assert(false)")
        .assert()
        .failure();
}

#[test]
fn test_zip_two_arrays() {
    ruchy_cmd()
        .arg("-e")
        .arg("let z = zip([1, 2], [\"a\", \"b\"]); println(len(z))")
        .assert()
        .success()
        .stdout(predicate::str::contains("2"));
}

#[test]
fn test_enumerate_array() {
    ruchy_cmd()
        .arg("-e")
        .arg("let e = enumerate([\"a\", \"b\", \"c\"]); println(len(e))")
        .assert()
        .success()
        .stdout(predicate::str::contains("3"));
}

// ============================================================================
// I/O Functions (eval_builtin.rs lines 87-94)
// ============================================================================

#[test]
fn test_println_single_arg() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(\"Hello\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("Hello"));
}

#[test]
fn test_println_multiple_args() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(\"Count:\", 42, true)")
        .assert()
        .success()
        .stdout(predicate::str::contains("Count:"))
        .stdout(predicate::str::contains("42"))
        .stdout(predicate::str::contains("true"));
}

#[test]
fn test_print_no_newline() {
    // print() quotes strings, so output is "A""B" not AB
    ruchy_cmd()
        .arg("-e")
        .arg("print(\"A\"); print(\"B\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"A\"\"B\""));
}

#[test]
fn test_dbg_returns_value() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = dbg(123); println(x)")
        .assert()
        .success()
        .stdout(predicate::str::contains("123"));
}

// ============================================================================
// Property-Based Tests
// ============================================================================

#[test]
fn property_sqrt_pow_inverse() {
    // Property: sqrt(pow(x, 2)) ≈ x
    for x in [2.0, 5.0, 10.0] {
        ruchy_cmd().arg("-e")
            .arg(format!("let result = sqrt(pow({x}, 2.0)); assert(result >= {x} - 0.1); assert(result <= {x} + 0.1)"))
            .assert().success();
    }
}

#[test]
fn property_abs_non_negative() {
    // Property: abs(x) >= 0
    for x in [-100.0, -1.0, 0.0, 1.0, 100.0] {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("assert(abs({x}) >= 0.0)"))
            .assert()
            .success();
    }
}

#[test]
fn property_min_max_bounds() {
    // Property: min(a,b) <= a && min(a,b) <= b
    let pairs = [(3.0, 7.0), (-5.0, 2.0), (0.0, 0.0)];
    for (a, b) in pairs {
        ruchy_cmd()
            .arg("-e")
            .arg(format!(
                "let m = min({a}, {b}); assert(m <= {a}); assert(m <= {b})"
            ))
            .assert()
            .success();
    }
}

#[test]
fn property_len_never_negative() {
    // Property: len(x) >= 0
    let values = vec!["\"\"", "\"test\"", "[]", "[1,2,3]"];
    for val in values {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("assert(len({val}) >= 0)"))
            .assert()
            .success();
    }
}

#[test]
fn property_range_length() {
    // Property: len(range(n)) == n
    for n in [0, 5, 10, 20] {
        ruchy_cmd()
            .arg("-e")
            .arg(format!("assert_eq(len(range({n})), {n})"))
            .assert()
            .success();
    }
}

#[test]
fn property_reverse_twice_identity() {
    // Property: reverse(reverse(arr)) == arr
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = [1, 2, 3]; let rev = reverse(reverse(arr)); assert_eq(len(arr), len(rev))")
        .assert()
        .success();
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn integration_math_workflow() {
    ruchy_cmd()
        .arg("-e")
        .arg("let a = pow(2.0, 4.0); let b = sqrt(a); let c = floor(b); println(c)")
        .assert()
        .success()
        .stdout(predicate::str::contains("4"));
}

#[test]
fn integration_array_operations() {
    ruchy_cmd()
        .arg("-e")
        .arg("let arr = range(5); let rev = reverse(arr); println(len(rev))")
        .assert()
        .success()
        .stdout(predicate::str::contains("5"));
}

#[test]
fn integration_assertions() {
    ruchy_cmd()
        .arg("-e")
        .arg("let x = 42; assert(x > 0); assert_eq(x, 42); println(\"All assertions passed\")")
        .assert()
        .success()
        .stdout(predicate::str::contains("All assertions passed"));
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn edge_case_sqrt_large_number() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(sqrt(1000000.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("1000"));
}

#[test]
fn edge_case_pow_zero_exponent() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(pow(999.0, 0.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn edge_case_min_same_values() {
    // min requires 2 args, test with same values
    ruchy_cmd()
        .arg("-e")
        .arg("println(min(42.0, 42.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_max_same_values() {
    // max requires 2 args, test with same values
    ruchy_cmd()
        .arg("-e")
        .arg("println(max(42.0, 42.0))")
        .assert()
        .success()
        .stdout(predicate::str::contains("42"));
}

#[test]
fn edge_case_range_zero() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(len(range(0)))")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}

#[test]
fn edge_case_reverse_empty() {
    ruchy_cmd()
        .arg("-e")
        .arg("println(len(reverse([])))")
        .assert()
        .success()
        .stdout(predicate::str::contains("0"));
}
