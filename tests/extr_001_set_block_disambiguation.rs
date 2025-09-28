//! EXTR-001: Set vs Block Disambiguation Tests
//!
//! These tests ensure proper disambiguation between:
//! - Block expressions: { statements }
//! - Set literals: {1, 2, 3}
//! - Object literals: {x: 1, y: 2}
//!
//! The core issue: {x} should be a block returning x, not a set containing x

use ruchy::backend::transpiler::Transpiler;
use ruchy::compile;
use ruchy::frontend::parser::Parser;

#[cfg(test)]
mod set_block_tests {
    use super::*;

    // ===== BLOCK TESTS - These should parse as blocks, not sets =====

    #[test]
    fn test_single_expr_block() {
        // {x} should be a block that returns x, NOT a set containing x
        let code = "fn test() -> i32 { 42 }";
        let result = compile(code);
        assert!(result.is_ok(), "Single expression block should compile");

        // Verify it doesn't generate HashSet
        let rust_code = result.unwrap();
        assert!(
            !rust_code.contains("HashSet"),
            "Should not generate HashSet for block"
        );
        assert!(
            rust_code.contains("fn test") && rust_code.contains("-> i32"),
            "Should generate function"
        );
    }

    #[test]
    fn test_function_body_block() {
        // Function bodies are always blocks
        let code = "fn add(a: i32, b: i32) -> i32 { a + b }";
        let result = compile(code);
        assert!(result.is_ok(), "Function body block should compile");

        let rust_code = result.unwrap();
        assert!(
            !rust_code.contains("HashSet"),
            "Function body should not be a set"
        );
    }

    #[test]
    fn test_nested_blocks() {
        // Nested blocks should work correctly
        let code = "fn test() -> i32 { { { 42 } } }";
        let result = compile(code);
        assert!(result.is_ok(), "Nested blocks should compile");

        let rust_code = result.unwrap();
        assert!(
            !rust_code.contains("HashSet"),
            "Nested blocks should not be sets"
        );
    }

    #[test]
    fn test_block_with_statements() {
        // Blocks with statements are clearly blocks
        let code = "fn test() -> i32 { let x = 10; x + 5 }";
        let result = compile(code);
        assert!(result.is_ok(), "Block with statements should compile");

        let rust_code = result.unwrap();
        assert!(
            !rust_code.contains("HashSet"),
            "Block with statements not a set"
        );
    }

    #[test]
    fn test_block_with_semicolon() {
        // Semicolon indicates block, not set
        let code = "fn test() { let x = { 5; }; }";
        let result = compile(code);
        assert!(result.is_ok(), "Block with semicolon should compile");

        let rust_code = result.unwrap();
        assert!(
            !rust_code.contains("HashSet"),
            "Semicolon means block not set"
        );
    }

    // ===== SET LITERAL TESTS - These should parse as sets =====

    #[test]
    fn test_empty_set() {
        // {} as a standalone expression could be empty set
        // But in function context it's an empty block
        let code = "let s = {};"; // Empty set literal
        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert!(ast.is_ok(), "Empty braces should parse");
    }

    #[test]
    fn test_set_with_multiple_elements() {
        // {1, 2, 3} is clearly a set literal
        let code = "let s = {1, 2, 3};";
        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert!(ast.is_ok(), "Set with multiple elements should parse");

        if let Ok(ast) = ast {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            if let Ok(rust_code) = result {
                let rust_str = rust_code.to_string();
                assert!(
                    rust_str.contains("HashSet") || rust_str.contains("vec![1, 2, 3]"),
                    "Should generate set/collection code"
                );
            }
        }
    }

    #[test]
    fn test_set_with_trailing_comma() {
        // {1, 2, 3,} with trailing comma is a set
        let code = "let s = {1, 2, 3,};";
        let mut parser = Parser::new(code);
        let ast = parser.parse();
        assert!(ast.is_ok(), "Set with trailing comma should parse");
    }

    // ===== DISAMBIGUATION TESTS - Context determines interpretation =====

    #[test]
    fn test_function_return_position() {
        // In return position, {expr} should be a block
        let code = "fn test() -> i32 { { 5 + 5 } }";
        let result = compile(code);
        assert!(result.is_ok(), "Block in return position should compile");

        let rust_code = result.unwrap();
        assert!(
            !rust_code.contains("HashSet"),
            "Return position means block"
        );
    }

    #[test]
    fn test_let_binding_rhs() {
        // In let binding RHS, could be either based on content
        // Single expr without comma = block
        let code1 = "fn test() { let x = { 42 }; }";
        let result1 = compile(code1);
        assert!(result1.is_ok(), "Let with single-expr block should compile");

        // Multiple exprs with comma = set
        let code2 = "fn test() { let x = {1, 2, 3}; }";
        let result2 = compile(code2);
        assert!(result2.is_ok(), "Let with set literal should compile");
    }

    #[test]
    fn test_if_condition_body() {
        // If/else bodies are always blocks
        let code = "fn test() -> i32 { if true { 42 } else { 0 } }";
        let result = compile(code);
        assert!(result.is_ok(), "If/else blocks should compile");

        let rust_code = result.unwrap();
        assert!(!rust_code.contains("HashSet"), "If/else bodies are blocks");
    }

    #[test]
    fn test_match_arm_body() {
        // Match arms use blocks
        let code = "fn test(x: i32) -> i32 { match x { 0 => { 42 } _ => { 0 } } }";
        let result = compile(code);
        assert!(result.is_ok(), "Match arm blocks should compile");

        let rust_code = result.unwrap();
        assert!(!rust_code.contains("HashSet"), "Match arms use blocks");
    }

    // ===== PARSER STATE TESTS - Verify parser makes correct decisions =====

    #[test]
    fn test_parser_function_body_decision() {
        let mut parser = Parser::new("fn f() { x }");
        let ast = parser.parse();
        assert!(ast.is_ok(), "Should parse function with block body");

        // Verify AST has Block, not Set
        if let Ok(ast) = ast {
            let s = format!("{ast:?}");
            assert!(
                s.contains("Block") || !s.contains("Set"),
                "Function body should be Block in AST"
            );
        }
    }

    #[test]
    fn test_parser_comma_decision() {
        // Comma after first element means set
        let mut parser = Parser::new("{1, 2}");
        let ast = parser.parse();
        assert!(ast.is_ok(), "Should parse set literal");

        if let Ok(ast) = ast {
            let s = format!("{ast:?}");
            assert!(
                s.contains("Set") || s.contains("[1, 2]"),
                "Comma-separated should be Set in AST"
            );
        }
    }

    #[test]
    fn test_parser_statement_decision() {
        // Statements mean block
        let mut parser = Parser::new("{ let x = 5; x }");
        let ast = parser.parse();
        assert!(ast.is_ok(), "Should parse block with let");

        if let Ok(ast) = ast {
            let s = format!("{ast:?}");
            assert!(
                s.contains("Block") || s.contains("Let"),
                "Let statement means Block in AST"
            );
        }
    }

    // ===== REGRESSION TESTS - Ensure fix doesn't break existing functionality =====

    #[test]
    fn test_factorial_still_works() {
        // From P0 tests - must continue working
        let code = r"
            fn factorial(n: i32) -> i32 {
                if n <= 1 { 1 } else { n * factorial(n - 1) }
            }
        ";
        let result = compile(code);
        assert!(result.is_ok(), "Factorial should still compile after fix");

        let rust_code = result.unwrap();
        assert!(
            !rust_code.contains("HashSet"),
            "Factorial should not use HashSet"
        );
    }

    #[test]
    fn test_match_still_works() {
        // From P0 tests - must continue working
        let code = r#"
            fn classify(n: i32) -> String {
                match n {
                    0 => "zero"
                    1 => "one"
                    _ => "many"
                }
            }
        "#;
        let result = compile(code);
        assert!(result.is_ok(), "Match should still compile after fix");

        let rust_code = result.unwrap();
        assert!(
            !rust_code.contains("HashSet"),
            "Match should not use HashSet"
        );
    }
}
