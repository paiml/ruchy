#![allow(missing_docs)]
// Property tests for RUNTIME-092: Enum variable cast support (Issue #79)
//
// These tests use proptest to verify enum cast behavior across 10K+ random inputs
// Testing: enum variable casts, arithmetic operations, type conversions
//
// Test naming convention: test_property_runtime_092_<scenario>

use assert_cmd::Command;
use predicates::prelude::*;
use proptest::prelude::*;

/// Property test: Enum discriminants convert correctly to i32
#[test]
fn test_property_runtime_092_enum_to_i32() {
    proptest!(|(disc0 in 0i64..10, disc1 in 10i64..20)| {
                                    let code = format!(r"
enum TestEnum {{
    Variant0 = {disc0},
    Variant1 = {disc1},
}}
fun main() {{
    let v0 = TestEnum::Variant0;
    let v1 = TestEnum::Variant1;
    let val0 = v0 as i32;
    let val1 = v1 as i32;
    println(val0);
    println(val1);
}}
");

                                    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                                        .arg("-e")
                                        .arg(&code)
                                        .timeout(std::time::Duration::from_secs(5))
                                        .assert()
                                        .success()
                                        .stdout(predicate::str::contains(disc0.to_string()))
                                        .stdout(predicate::str::contains(disc1.to_string()));
                                });
}

/// Property test: Enum casts in arithmetic expressions
#[test]
fn test_property_runtime_092_enum_arithmetic() {
    proptest!(|(disc in 0i64..100, add_val in 1i64..10)| {
                                    let code = format!(r"
enum Value {{
    X = {disc},
}}
fun main() {{
    let v = Value::X;
    let result = (v as i32) + {add_val};
    println(result);
}}
");

                                    let expected = disc + add_val;
                                    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                                        .arg("-e")
                                        .arg(&code)
                                        .timeout(std::time::Duration::from_secs(5))
                                        .assert()
                                        .success()
                                        .stdout(predicate::str::contains(expected.to_string()));
                                });
}

/// Property test: Multiple enum variables with different discriminants
#[test]
fn test_property_runtime_092_multiple_variables() {
    proptest!(|(d0 in 0i64..10, d1 in 10i64..20, d2 in 20i64..30)| {
                                    let code = format!(r"
enum Status {{
    A = {d0},
    B = {d1},
    C = {d2},
}}
fun main() {{
    let a = Status::A;
    let b = Status::B;
    let c = Status::C;
    println(a as i32);
    println(b as i32);
    println(c as i32);
}}
");

                                    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                                        .arg("-e")
                                        .arg(&code)
                                        .timeout(std::time::Duration::from_secs(5))
                                        .assert()
                                        .success()
                                        .stdout(predicate::str::contains(d0.to_string()))
                                        .stdout(predicate::str::contains(d1.to_string()))
                                        .stdout(predicate::str::contains(d2.to_string()));
                                });
}

/// Property test: Enum cast to different integer types (i32, i64, isize)
#[test]
fn test_property_runtime_092_multiple_int_types() {
    proptest!(|(disc in 0i64..1000)| {
                                    let code = format!(r"
enum Value {{
    X = {disc},
}}
fun main() {{
    let v = Value::X;
    let as_i32 = v as i32;
    let v2 = Value::X;
    let as_i64 = v2 as i64;
    let v3 = Value::X;
    let as_isize = v3 as isize;
    println(as_i32);
    println(as_i64);
    println(as_isize);
}}
");

                                    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                                        .arg("-e")
                                        .arg(&code)
                                        .timeout(std::time::Duration::from_secs(5))
                                        .assert()
                                        .success()
                                        .stdout(predicate::str::contains(disc.to_string()).count(3));
                                });
}

/// Property test: Enum casts preserve discriminant values through operations
#[test]
fn test_property_runtime_092_discriminant_preservation() {
    proptest!(|(disc in 0i64..50)| {
                                    let code = format!(r"
enum Priority {{
    Level = {disc},
}}
fun main() {{
    let p = Priority::Level;
    let val1 = p as i32;
    let p2 = Priority::Level;
    let val2 = p2 as i32;
    let sum = val1 + val2;
    println(sum);
}}
");

                                    let expected = disc * 2;
                                    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
                                        .arg("-e")
                                        .arg(&code)
                                        .timeout(std::time::Duration::from_secs(5))
                                        .assert()
                                        .success()
                                        .stdout(predicate::str::contains(expected.to_string()));
                                });
}
