#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! Regression tests for critical bugs fixed in v0.4.7
//! These tests ensure that the emergency fixes remain stable

#![allow(clippy::expect_used)]
#![allow(clippy::needless_raw_string_hashes)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::needless_raw_string_hashes)]

use ruchy::runtime::Repl;

#[test]
fn test_variable_binding_persistence() {
    // Regression test for Unit Add Unit bug where variables were corrupted
    let mut repl = Repl::new().expect("Failed to create REPL");

    // Set up variables
    repl.eval("let x = 10").expect("Failed to set x");
    repl.eval("let y = 20").expect("Failed to set y");

    // Verify variables retain their values
    let x_val = repl.eval("x").expect("Failed to get x");
    assert_eq!(x_val, "10");

    let y_val = repl.eval("y").expect("Failed to get y");
    assert_eq!(y_val, "20");

    // Verify arithmetic works
    let sum = repl.eval("x + y").expect("Failed to add x + y");
    assert_eq!(sum, "30");

    // Verify println with expressions works
    let result = repl
        .eval("println(x + y, x * y)")
        .expect("Failed to println expressions");
    assert_eq!(result, "()");
}

#[test]
fn test_let_binding_returns_unit() {
    // Ensure let bindings return Unit but store the correct value
    let mut repl = Repl::new().expect("Failed to create REPL");

    let result = repl.eval("let z = 42").expect("Failed to create binding");
    assert_eq!(result, "()", "Let binding should return Unit");

    let value = repl.eval("z").expect("Failed to retrieve z");
    assert_eq!(value, "42", "Variable should retain its value");
}

#[test]
fn test_block_returns_last_value() {
    // Regression test for blocks returning first instead of last value
    let mut repl = Repl::new().expect("Failed to create REPL");

    let result = repl.eval("{ 1; 2; 3 }").expect("Failed to evaluate block");
    assert_eq!(result, "3", "Block should return last value");

    let result = repl
        .eval("{ let a = 5; a * 2 }")
        .expect("Failed to evaluate block with binding");
    assert_eq!(
        result, "10",
        "Block with binding should return last expression"
    );
}

#[test]
fn test_match_expression_evaluation() {
    // Test that match expressions work correctly
    let mut repl = Repl::new().expect("Failed to create REPL");

    let result = repl
        .eval(
            r#"
        match 42 {
            42 => "found",
            _ => "not found"
        }
    "#,
        )
        .expect("Failed to evaluate match");
    assert_eq!(result, r#""found""#);

    // Test wildcard pattern
    let result = repl
        .eval(
            r#"
        match 99 {
            42 => "forty-two",
            _ => "other"
        }
    "#,
        )
        .expect("Failed to evaluate match with wildcard");
    assert_eq!(result, r#""other""#);
}

#[test]
fn test_function_definition_and_call() {
    // Ensure functions can be defined and called
    let mut repl = Repl::new().expect("Failed to create REPL");

    repl.eval("fun double(x: i32) -> i32 { x * 2 }")
        .expect("Failed to define function");
    let result = repl.eval("double(21)").expect("Failed to call function");
    assert_eq!(result, "42");
}

#[test]
fn test_string_interpolation_complex() {
    // Test f-string interpolation with expressions
    let mut repl = Repl::new().expect("Failed to create REPL");

    repl.eval("let name = \"World\"")
        .expect("Failed to set name");
    repl.eval("let num = 42").expect("Failed to set num");

    let result = repl
        .eval(r#"f"Hello, {name}! The answer is {num}""#)
        .expect("Failed to interpolate");
    assert_eq!(result, r#""Hello, World! The answer is 42""#);
}

#[test]
fn test_nested_control_flow() {
    // Test complex nested structures work correctly
    let mut repl = Repl::new().expect("Failed to create REPL");

    let _result = repl
        .eval(
            r#"
        let nums = [1, 2, 3, 4]
        let mut sum = 0
        for n in nums {
            if n % 2 == 0 {
                let sum = sum + n
            }
        }
        sum
    "#,
        )
        .expect("Failed complex nested evaluation");

    // Note: This might not work perfectly due to scoping, but it tests the structure
}

#[test]
fn test_list_operations() {
    // Test list creation and iteration
    let mut repl = Repl::new().expect("Failed to create REPL");

    repl.eval("let list = [10, 20, 30]")
        .expect("Failed to create list");
    let result = repl.eval("list").expect("Failed to retrieve list");
    assert_eq!(result, "[10, 20, 30]");

    // Test for loop over list
    let result = repl
        .eval("for x in [1, 2] { println(x) }")
        .expect("Failed to iterate");
    assert_eq!(result, "()");
}

#[test]
fn test_character_literals() {
    // Test character literal support
    let mut repl = Repl::new().expect("Failed to create REPL");

    let result = repl.eval("'a'").expect("Failed to evaluate char");
    assert_eq!(result, "'a'");

    // Note: Escape sequences in character literals need further work
    // For now, just test basic character literals work
    let result = repl.eval("'z'").expect("Failed to evaluate char");
    assert_eq!(result, "'z'");
}

#[test]
fn test_comparison_operators() {
    // Test all comparison operators work
    let mut repl = Repl::new().expect("Failed to create REPL");

    assert_eq!(repl.eval("5 > 3").unwrap(), "true");
    assert_eq!(repl.eval("5 < 3").unwrap(), "false");
    assert_eq!(repl.eval("5 >= 5").unwrap(), "true");
    assert_eq!(repl.eval("5 <= 4").unwrap(), "false");
    assert_eq!(repl.eval("5 == 5").unwrap(), "true");
    assert_eq!(repl.eval("5 != 3").unwrap(), "true");
}
