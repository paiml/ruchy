//! COMPLETE LANGUAGE RESTORATION - TDD + TDG Sprint
//! Systematically restore ALL missing language features
//! Target: 100% test pass rate with TDG A- grade compliance

use ruchy::{Parser, Transpiler};

// ============================================================================
// ITERATION CONSTRUCTS
// ============================================================================

#[test]
fn test_while_loop_parsing() {
    let code = r#"while x < 10 {
        x = x + 1
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse while loop: {:?}", 
        result.err()
    );
}

#[test]
fn test_for_loop_parsing() {
    let code = "for i in 0..10 { println(i) }";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse for loop: {:?}", 
        result.err()
    );
}

#[test]
fn test_for_loop_with_iterator() {
    let code = "for item in list { process(item) }";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse for-in loop: {:?}", 
        result.err()
    );
}

// ============================================================================
// DATA STRUCTURES
// ============================================================================

#[test]
fn test_list_literal_parsing() {
    let code = "[1, 2, 3, 4, 5]";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse list literal: {:?}", 
        result.err()
    );
}

#[test]
fn test_empty_list_literal() {
    let code = "[]";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse empty list: {:?}", 
        result.err()
    );
}

#[test]
fn test_nested_list_literal() {
    let code = "[[1, 2], [3, 4], [5, 6]]";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse nested lists: {:?}", 
        result.err()
    );
}

// ============================================================================
// LAMBDA/CLOSURE SYNTAX
// ============================================================================

#[test]
fn test_lambda_expression() {
    let code = "|x| x + 1";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse lambda: {:?}", 
        result.err()
    );
}

#[test]
fn test_lambda_with_multiple_params() {
    let code = "|x, y| x + y";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse multi-param lambda: {:?}", 
        result.err()
    );
}

#[test]
fn test_lambda_with_fat_arrow() {
    let code = "x => x * 2";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse fat arrow lambda: {:?}", 
        result.err()
    );
}

// ============================================================================
// TYPE SYSTEM
// ============================================================================

#[test]
fn test_struct_definition() {
    let code = r#"struct Point {
        x: i32,
        y: i32
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse struct: {:?}", 
        result.err()
    );
}

#[test]
fn test_trait_definition() {
    let code = r#"trait Display {
        fun show(self) -> str
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse trait: {:?}", 
        result.err()
    );
}

#[test]
fn test_impl_block() {
    let code = r#"impl Display for Point {
        fun show(self) -> str {
            format("{}, {}", self.x, self.y)
        }
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse impl block: {:?}", 
        result.err()
    );
}

// ============================================================================
// MODULE SYSTEM
// ============================================================================

#[test]
fn test_import_statement() {
    let code = "import std::io";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse import: {:?}", 
        result.err()
    );
}

#[test]
fn test_use_statement() {
    let code = "use std::collections::HashMap";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse use statement: {:?}", 
        result.err()
    );
}

#[test]
fn test_pub_function() {
    let code = "pub fun add(x: i32, y: i32) -> i32 { x + y }";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse public function: {:?}", 
        result.err()
    );
}

// ============================================================================
// ADVANCED FEATURES
// ============================================================================

#[test]
fn test_string_interpolation() {
    let code = r#"f"Hello {name}, you are {age} years old""#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse f-string: {:?}", 
        result.err()
    );
}

#[test]
fn test_dataframe_literal() {
    let code = r#"df![
        "name" => ["Alice", "Bob"],
        "age" => [30, 25]
    ]"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse dataframe literal: {:?}", 
        result.err()
    );
}

#[test]
fn test_actor_definition() {
    let code = r#"actor Counter {
        state count: i32 = 0
        
        receive increment() {
            self.count += 1
        }
    }"#;
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse actor: {:?}", 
        result.err()
    );
}

// ============================================================================
// SPECIAL SYNTAX
// ============================================================================

#[test]
fn test_unit_type() {
    let code = "()";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse unit type: {:?}", 
        result.err()
    );
}

#[test]
fn test_pipeline_operator() {
    let code = "data |> filter |> map |> collect";
    
    let mut parser = Parser::new(code);
    let result = parser.parse();
    
    assert!(result.is_ok(), 
        "Should parse pipeline: {:?}", 
        result.err()
    );
}

// ============================================================================
// TRANSPILATION TESTS
// ============================================================================

#[test]
fn test_while_loop_transpilation() {
    let code = "while x < 10 { x = x + 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse for transpilation");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), 
        "Should transpile while loop: {:?}", 
        result.err()
    );
    
    let generated = result.unwrap().to_string();
    assert!(generated.contains("while"), 
        "Should contain while keyword: {}", generated);
}

#[test]
fn test_for_loop_transpilation() {
    let code = "for i in 0..10 { println(i) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse for transpilation");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), 
        "Should transpile for loop: {:?}", 
        result.err()
    );
    
    let generated = result.unwrap().to_string();
    assert!(generated.contains("for"), 
        "Should contain for keyword: {}", generated);
}

#[test]
fn test_list_literal_transpilation() {
    let code = "[1, 2, 3]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse for transpilation");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast);
    
    assert!(result.is_ok(), 
        "Should transpile list literal: {:?}", 
        result.err()
    );
    
    let generated = result.unwrap().to_string();
    assert!(generated.contains("vec!"), 
        "Should use vec! macro: {}", generated);
}