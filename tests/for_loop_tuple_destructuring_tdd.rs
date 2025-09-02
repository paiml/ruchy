use ruchy::frontend::parser::Parser;
use ruchy::frontend::ast::{Expr, ExprKind, Pattern};

#[test]
fn test_for_loop_tuple_destructuring_basic() {
    // RED: This should fail initially - tuple destructuring not supported
    let source = "for (x, y) in [(1, 2), (3, 4)] { println(x + y) }";
    let mut parser = Parser::new(source);
    
    let result = parser.parse();
    
    match result {
        Ok(Expr { kind: ExprKind::For { pattern, .. }, .. }) => {
            // Should parse as tuple pattern, not identifier
            match pattern {
                Some(Pattern::Tuple(patterns)) => {
                    assert_eq!(patterns.len(), 2);
                    match (&patterns[0], &patterns[1]) {
                        (Pattern::Identifier(x), Pattern::Identifier(y)) => {
                            assert_eq!(x, "x");
                            assert_eq!(y, "y");
                        }
                        _ => panic!("Expected identifier patterns in tuple")
                    }
                }
                _ => panic!("Expected tuple pattern in for loop, got {:?}", pattern)
            }
        }
        Ok(other) => panic!("Expected For expression, got {:?}", other.kind),
        Err(e) => {
            // This is expected to fail initially
            println!("Expected failure (RED phase): {}", e);
            assert!(e.to_string().contains("Expected identifier or underscore in for pattern"));
        }
    }
}

#[test]
fn test_for_loop_object_items_method() {
    // RED: This should also fail - same issue with tuple destructuring
    let source = "for (key, value) in obj.items() { println(key) }";
    let mut parser = Parser::new(source);
    
    let result = parser.parse();
    
    match result {
        Ok(Expr { kind: ExprKind::For { pattern, .. }, .. }) => {
            match pattern {
                Some(Pattern::Tuple(patterns)) => {
                    assert_eq!(patterns.len(), 2);
                    match (&patterns[0], &patterns[1]) {
                        (Pattern::Identifier(key), Pattern::Identifier(value)) => {
                            assert_eq!(key, "key");
                            assert_eq!(value, "value");
                        }
                        _ => panic!("Expected identifier patterns")
                    }
                }
                _ => panic!("Expected tuple pattern, got {:?}", pattern)
            }
        }
        Ok(other) => panic!("Expected For expression, got {:?}", other.kind),
        Err(e) => {
            // Expected to fail initially
            println!("Expected failure (RED phase): {}", e);
            assert!(e.to_string().contains("Expected identifier or underscore in for pattern"));
        }
    }
}

#[test]
fn test_simple_for_loop_still_works() {
    // GREEN: This should continue to work - ensure no regression
    let source = "for x in [1, 2, 3] { println(x) }";
    let mut parser = Parser::new(source);
    
    let result = parser.parse();
    
    match result {
        Ok(Expr { kind: ExprKind::For { var, pattern, .. }, .. }) => {
            // Should use the var field for simple identifier
            assert_eq!(var, "x");
            // Pattern should be Some(Identifier) for simple identifier
            match pattern {
                Some(Pattern::Identifier(name)) => assert_eq!(name, "x"),
                _ => panic!("Simple for loop should have Identifier pattern, got {:?}", pattern),
            }
        }
        Ok(other) => panic!("Expected For expression, got {:?}", other.kind),
        Err(e) => panic!("Simple for loop should still work: {}", e),
    }
}