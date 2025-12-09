#![allow(missing_docs)]
// DEFECT-CLOSURE-RETURN: Functions returning closures fail with type inference error
// ROOT CAUSE: Transpiler defaults to `-> i32` for any non-unit expression
// FIX: Detect when function body is a closure and generate `impl Fn` return type
// EXTREME TDD: RED→GREEN→REFACTOR

use assert_cmd::Command;
use predicates::prelude::*;

/// Helper to get ruchy binary command
fn ruchy_cmd() -> Command {
    assert_cmd::cargo::cargo_bin_cmd!("ruchy")
}

// RED: This test MUST fail before the fix
#[test]
#[ignore = "RED phase: closure return type inference not yet implemented - DEFECT-CLOSURE-RETURN"]
fn test_defect_closure_return_simple_adder() {
    let temp_file = std::env::temp_dir().join("defect_closure_return_adder.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn main() {
    fn make_adder(n) {
        |x| { x + n }
    }

    let add_five = make_adder(5)
    println(add_five(10))
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("15"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
#[ignore = "RED phase: closure return type inference not yet implemented - DEFECT-CLOSURE-RETURN"]
fn test_defect_closure_return_multiplier() {
    let temp_file = std::env::temp_dir().join("defect_closure_return_mult.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn main() {
    fn make_multiplier(factor) {
        |x| { x * factor }
    }

    let times_three = make_multiplier(3)
    println(times_three(7))
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("21"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
#[ignore = "RED phase: closure return type inference not yet implemented - DEFECT-CLOSURE-RETURN"]
fn test_defect_closure_return_counter() {
    let temp_file = std::env::temp_dir().join("defect_closure_return_counter.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn main() {
    fn make_counter(start) {
        |increment| { start + increment }
    }

    let counter = make_counter(10)
    println(counter(5))
    println(counter(3))
}
",
    )
    .unwrap();

    ruchy_cmd()
        .arg("run")
        .arg(&temp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("15"))
        .stdout(predicate::str::contains("13"));

    std::fs::remove_file(&temp_file).ok();
}

#[test]
#[ignore = "RED phase: closure return type inference not yet implemented - DEFECT-CLOSURE-RETURN"]
fn test_defect_closure_return_transpile() {
    // Verify transpiler generates correct Rust code
    let temp_file = std::env::temp_dir().join("defect_closure_return_transpile.ruchy");
    std::fs::write(
        &temp_file,
        r"
fn make_adder(n) {
    |x| { x + n }
}
",
    )
    .unwrap();

    let output = ruchy_cmd()
        .arg("transpile")
        .arg(&temp_file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let transpiled = String::from_utf8(output).unwrap();

    // Should contain impl Fn, not -> i32
    assert!(
        transpiled.contains("impl Fn"),
        "Transpiled code should contain 'impl Fn', got: {transpiled}"
    );

    // Should NOT have incorrect -> i32 return type
    assert!(
        !transpiled.contains("make_adder (n : & str) -> i32"),
        "Transpiled code should not have incorrect i32 return type, got: {transpiled}"
    );

    std::fs::remove_file(&temp_file).ok();
}
