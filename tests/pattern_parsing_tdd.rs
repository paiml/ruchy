use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::{Expr, ExprKind, Pattern};

#[test]
fn test_tuple_pattern_parsing() {
    let source = "let (x, y) = (1, 2)";
    let mut parser = Parser::new(source);
    
    // This should parse as let with tuple pattern
    let result = parser.parse();
    
    match result {
        Ok(Expr { kind: ExprKind::LetPattern { pattern, .. }, .. }) => {
            // Should parse as LetPattern for tuple destructuring
            match pattern {
                Pattern::Tuple(patterns) => {
                    assert_eq!(patterns.len(), 2);
                    match (&patterns[0], &patterns[1]) {
                        (Pattern::Identifier(x), Pattern::Identifier(y)) => {
                            assert_eq!(x, "x");
                            assert_eq!(y, "y");
                        }
                        _ => panic!("Expected identifier patterns")
                    }
                }
                _ => panic!("Expected tuple pattern")
            }
        }
        Ok(other) => panic!("Expected LetPattern expression, got {:?}", other.kind),
        Err(e) => panic!("Parse error: {}", e),
    }
}

#[test]  
fn test_simple_let_statement() {
    let source = "let x = 5";
    let mut parser = Parser::new(source);
    
    // This should parse as simple let
    let result = parser.parse();
    
    match result {
        Ok(Expr { kind: ExprKind::Let { name, .. }, .. }) => {
            assert_eq!(name, "x");
        }
        Ok(other) => panic!("Expected Let expression, got {:?}", other.kind),
        Err(e) => panic!("Parse error: {}", e),
    }
}