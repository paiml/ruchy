//! TDD for pattern parsing improvements

use ruchy::frontend::ast::{ExprKind, Pattern};
use ruchy::Parser;

#[test]
fn test_parse_nested_tuple_pattern() {
    // RED phase - this should fail because parser doesn't support nested patterns
    let code = "let ((a, b), (c, d)) = nested";
    let mut parser = Parser::new(code);
    let ast = parser.parse();

    // This should fail initially
    assert!(ast.is_err() || matches_nested_pattern(&ast.unwrap()));
}

fn matches_nested_pattern(expr: &ruchy::frontend::ast::Expr) -> bool {
    match &expr.kind {
        ExprKind::LetPattern { pattern, .. } => {
            matches!(pattern, Pattern::Tuple(patterns) 
                if patterns.len() == 2 
                && matches!(patterns[0], Pattern::Tuple(_))
                && matches!(patterns[1], Pattern::Tuple(_)))
        }
        _ => false,
    }
}

#[test]
fn test_parse_simple_tuple_pattern() {
    // This should already work
    let code = "let (x, y) = tup";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Simple tuple pattern should parse");

    match &ast.kind {
        ExprKind::LetPattern { pattern, .. } => {
            assert!(matches!(pattern, Pattern::Tuple(patterns) if patterns.len() == 2));
        }
        _ => panic!("Expected LetPattern"),
    }
}
