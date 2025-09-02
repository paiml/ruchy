#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
//! Basic test for reference operator parsing

#![allow(clippy::expect_used)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::panic)]
#![allow(clippy::single_char_pattern)]

use ruchy::{Parser, Transpiler};

#[test]
fn test_reference_operator_basic_parsing() {
    // Test just parsing without evaluation
    let input = "&42";
    let mut parser = Parser::new(input);
    let expr = parser
        .parse()
        .expect("Failed to parse reference expression");

    // Verify it parsed as a unary reference expression
    match &expr.kind {
        ruchy::ExprKind::Unary { op, operand: _ } => {
            assert_eq!(
                *op,
                ruchy::UnaryOp::Reference,
                "Should parse as reference operator"
            );
        }
        _ => panic!("Expected unary reference expression, got: {:?}", expr.kind),
    }
}

#[test]
fn test_reference_operator_transpilation() {
    let input = "&42";
    let mut parser = Parser::new(input);
    let expr = parser
        .parse()
        .expect("Failed to parse reference expression");

    let mut transpiler = Transpiler::new();
    let result = transpiler.transpile(&expr);
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

#[test]
fn test_reference_vs_bitwise_basic() {
    // Test that & can be parsed in different contexts

    // Bitwise AND
    let input = "5 & 3";
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse bitwise AND");
    match &expr.kind {
        ruchy::ExprKind::Binary { op, .. } => {
            assert_eq!(
                *op,
                ruchy::BinaryOp::BitwiseAnd,
                "Should parse as bitwise AND"
            );
        }
        _ => panic!("Expected binary expression, got: {:?}", expr.kind),
    }

    // Reference (unary)
    let input = "&5";
    let mut parser = Parser::new(input);
    let expr = parser.parse().expect("Failed to parse reference");
    match &expr.kind {
        ruchy::ExprKind::Unary { op, .. } => {
            assert_eq!(*op, ruchy::UnaryOp::Reference, "Should parse as reference");
        }
        _ => panic!("Expected unary expression, got: {:?}", expr.kind),
    }
}
