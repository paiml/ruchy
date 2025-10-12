//! String Interpolation REPL Bug Fix Test
//! EXTREME TDD: RED phase - This test MUST fail before the fix

use ruchy::runtime::{Repl, Value};
use std::sync::Arc;

#[test]
fn test_repl_string_interpolation_no_quotes() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    // Set a string variable
    repl.evaluate_expr_str(r#"let name = "World""#, None)
        .unwrap();

    // Test interpolation
    let result = repl.evaluate_expr_str(r#"f"Hello {name}""#, None).unwrap();

    // Should be "Hello World" not "Hello \"World\""
    assert_eq!(
        result,
        Value::String(Arc::from("Hello World")),
        "REPL string interpolation should not add quotes to string variables"
    );
}

#[test]
fn test_repl_string_interpolation_with_integer() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();

    repl.evaluate_expr_str("let count = 42", None).unwrap();
    let result = repl
        .evaluate_expr_str(r#"f"Count: {count}""#, None)
        .unwrap();

    assert_eq!(
        result,
        Value::String(Arc::from("Count: 42")),
        "REPL should interpolate integers correctly"
    );
}
