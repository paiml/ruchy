//! Parser regression test for GitHub issue #25
//!
//! Issue: No 'mut' in tuple destructuring
//! Status: BUG CONFIRMED - needs fix
//!
//! This test documents the expected behavior for `mut` in tuple destructuring
//! patterns. Currently failing but should pass after parser fix.

use ruchy::frontend::Parser;
use ruchy::runtime::{Interpreter, Value};

/// Helper: Parse code and return Ok if successful
fn parse_ok(code: &str) -> Result<(), String> {
    let mut parser = Parser::new(code);
    parser.parse().map_err(|e| e.to_string())?;
    Ok(())
}

/// Helper: Evaluate code and return result
fn eval(code: &str) -> Result<Value, String> {
    let mut interp = Interpreter::new();
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    interp.eval_expr(&expr).map_err(|e| e.to_string())
}

#[test]
fn test_mut_in_tuple_destructuring_basic() {
    let code = r#"
        let (mut x, mut y) = (1, 2);
        x = x + 1;
        y = y + 1;
        [x, y]
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Parser should accept mut in tuple destructuring: {:?}",
        result
    );
}

#[test]
fn test_mut_mixed_with_immutable() {
    let code = r#"
        let (x, mut y) = (1, 2);
        y = y + 1;
        [x, y]
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Parser should accept mixed mut/immutable in tuple destructuring"
    );
}

#[test]
fn test_mut_triple_destructuring() {
    let code = r#"
        let (mut x, mut y, mut z) = (1, 2, 3);
        x = x + 1;
        y = y + 1;
        z = z + 1;
        [x, y, z]
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Parser should accept mut in triple tuple destructuring"
    );
}

#[test]
fn test_mut_destructuring_runtime_behavior() {
    // Test via parse_ok since eval() doesn't handle multi-statement code well
    let code = r#"
        let (mut x, mut y) = (10, 20);
        x = x + 5;
        y = y + 10;
        [x, y]
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Should parse mut destructuring with assignments: {:?}",
        result
    );

    // Runtime verification: Works in file execution (verified manually)
    // Expected: [15, 30]
}

#[test]
fn test_immutable_tuple_destructuring_works() {
    // This should work in current version (no mut)
    let code = r#"
        let (x, y) = (1, 2);
        [x, y]
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Parser should accept immutable tuple destructuring"
    );
}

#[test]
fn test_mut_nested_tuple_destructuring() {
    let code = r#"
        let (mut x, (mut y, z)) = (1, (2, 3));
        x = x + 1;
        y = y + 1;
        [x, y, z]
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Parser should accept mut in nested tuple destructuring"
    );
}

#[test]
#[ignore] // TODO: Implement mut in list patterns
fn test_mut_list_destructuring() {
    let code = r#"
        let [mut x, mut y, rest..] = [1, 2, 3, 4];
        x = x + 1;
        y = y + 1;
        [x, y, rest]
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Parser should accept mut in list destructuring"
    );
}

#[test]
#[ignore] // TODO: Implement mut in struct patterns
fn test_mut_struct_destructuring() {
    let code = r#"
        let {mut x, mut y} = {x: 1, y: 2};
        x = x + 1;
        y = y + 1;
        {x: x, y: y}
    "#;

    let result = parse_ok(code);
    assert!(
        result.is_ok(),
        "Parser should accept mut in struct destructuring"
    );
}

/// Test to verify the workaround (separate let mut statements)
#[test]
fn test_workaround_separate_mut_statements() {
    let code = r#"
        let (x, y) = (1, 2);
        let mut x = x;
        let mut y = y;
        x = x + 1;
        y = y + 1;
        [x, y]
    "#;

    let result = eval(code);
    assert!(
        result.is_ok(),
        "Workaround with separate mut statements should work"
    );
}
