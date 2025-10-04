#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! Tests for reference operator (&) functionality

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::print_stdout)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::single_char_pattern)]

use ruchy::runtime::Repl;
use std::env;

#[test]
fn test_reference_operator_parsing() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Check that reference operator can be parsed
    let result = repl.eval("&42");
    if let Err(e) = &result {
        println!("Error parsing &42: {}", e);
    }
    assert!(
        result.is_ok(),
        "Reference operator should parse successfully: {:?}",
        result
    );

    // Check reference with variable
    repl.eval("let x = 10").unwrap();
    let result = repl.eval("&x");
    if let Err(e) = &result {
        println!("Error parsing &x: {}", e);
    }
    assert!(
        result.is_ok(),
        "Reference to variable should work: {:?}",
        result
    );
}

#[test]
fn test_reference_operator_with_expressions() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Check reference with more complex expressions
    let result = repl.eval("&(1 + 2)");
    assert!(result.is_ok(), "Reference to expression should work");

    // Check reference with parenthesized expression
    let result = repl.eval("&(100)");
    assert!(
        result.is_ok(),
        "Reference to parenthesized value should work"
    );
}

#[test]
fn test_reference_operator_priority() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Check that reference operator has correct precedence as unary operator
    let result = repl.eval("&!true");
    assert!(
        result.is_ok(),
        "Reference should work with other unary operators"
    );

    let result = repl.eval("&-42");
    assert!(result.is_ok(), "Reference should work with negation");
}

#[test]
fn test_reference_vs_bitwise_and() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Check that & still works as bitwise AND in binary context
    let result = repl.eval("5 & 3");
    if let Err(e) = &result {
        println!("Error evaluating 5 & 3: {}", e);
    }
    assert!(
        result.is_ok(),
        "Bitwise AND should still work: {:?}",
        result
    );

    // Check that context determines interpretation
    let result = repl.eval("&5 & 3");
    assert!(result.is_ok(), "Reference and bitwise AND should coexist");
}

#[test]
fn test_reference_operator_transpilation() {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::ast::{Expr, ExprKind, Literal, Span, UnaryOp};

    let mut transpiler = Transpiler::new();

    // Create a reference expression manually
    let operand = Box::new(Expr::new(
        ExprKind::Literal(Literal::Integer(42)),
        Span::new(0, 2),
    ));

    let ref_expr = Expr::new(
        ExprKind::Unary {
            op: UnaryOp::Reference,
            operand,
        },
        Span::new(0, 3),
    );

    // Check transpilation
    let result = transpiler.transpile(&ref_expr);
    assert!(result.is_ok(), "Reference expression should transpile");

    let rust_code = result.unwrap().to_string();
    assert!(
        rust_code.contains("&"),
        "Transpiled code should contain reference operator"
    );
    assert!(
        rust_code.contains("42"),
        "Transpiled code should contain the operand"
    );
}
