//! TDD Test Suite for Book Compatibility Restoration
//! Target: 100% ruchy-book compatibility (from 68.6%)

use ruchy::{Parser, Transpiler};

// ============================================================================
// MUTABLE BINDINGS (let mut)
// ============================================================================

#[test]
fn test_let_mut_parsing() {
    let code = "let mut x = 5";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse let mut: {:?}", 
        result.err()
    );
}

#[test]
fn test_let_mut_with_reassignment() {
    let code = r#"
        let mut count = 0
        count = count + 1
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse mutable reassignment: {:?}", 
        result.err()
    );
}

// ============================================================================
// CONTROL FLOW EDGE CASES
// ============================================================================

#[test]
fn test_return_in_if_statement() {
    let code = r#"
        if x > 0 return x
        return 0
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse return in if without braces: {:?}", 
        result.err()
    );
}

#[test]
fn test_early_return() {
    let code = r#"
        fun check(x) {
            if x < 0 {
                return "negative"
            }
            return "positive"
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse early return: {:?}", 
        result.err()
    );
}

// ============================================================================
// LIST COMPREHENSIONS
// ============================================================================

#[test]
fn test_list_comprehension_basic() {
    let code = "[x * 2 for x in [1, 2, 3]]";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse list comprehension: {:?}", 
        result.err()
    );
}

#[test]
fn test_list_comprehension_with_filter() {
    let code = "[x for x in numbers if x > 0]";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse list comprehension with filter: {:?}", 
        result.err()
    );
}

// ============================================================================
// GENERIC TYPES
// ============================================================================

#[test]
fn test_generic_struct() {
    let code = r#"
        struct Box<T> {
            value: T
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse generic struct: {:?}", 
        result.err()
    );
}

#[test]
fn test_generic_function() {
    let code = r#"
        fun identity<T>(x: T) -> T {
            x
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse generic function: {:?}", 
        result.err()
    );
}

// ============================================================================
// ENUM DEFINITIONS
// ============================================================================

#[test]
fn test_enum_definition() {
    let code = r#"
        enum Option<T> {
            Some(T),
            None
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse enum: {:?}", 
        result.err()
    );
}

#[test]
fn test_enum_with_multiple_variants() {
    let code = r#"
        enum Result<T, E> {
            Ok(T),
            Err(E)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse enum with multiple generic params: {:?}", 
        result.err()
    );
}

// ============================================================================
// ASSIGNMENT OPERATORS
// ============================================================================

#[test]
fn test_compound_assignment() {
    let code = r#"
        x += 1
        y *= 2
        z -= 3
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse compound assignment: {:?}", 
        result.err()
    );
}

// ============================================================================
// BREAK AND CONTINUE
// ============================================================================

#[test]
fn test_break_in_loop() {
    let code = r#"
        while true {
            if done break
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse break: {:?}", 
        result.err()
    );
}

#[test]
fn test_continue_in_loop() {
    let code = r#"
        for i in 0..10 {
            if i % 2 == 0 continue
            println(i)
        }
    "#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse continue: {:?}", 
        result.err()
    );
}