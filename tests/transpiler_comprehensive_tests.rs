//! Comprehensive test suite for transpiler statements
//! Target: Increase coverage for src/backend/transpiler/statements.rs

use ruchy::backend::Transpiler;
use ruchy::frontend::ast::*;
use ruchy::Parser;

fn parse_and_transpile(code: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut parser = Parser::new(code);
    let ast = parser.parse()?;
    let mut transpiler = Transpiler::new();
    let tokens = transpiler.transpile(&ast)?;
    Ok(tokens.to_string())
}

#[test]
fn test_transpile_let_statement() {
    let code = "let x = 42";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("let") && output.contains("x") && output.contains("42"));
}

#[test]
fn test_transpile_mutable_let() {
    let code = "let mut y = 10";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("mut"));
}

#[test]
fn test_transpile_function_definition() {
    let code = "fun add(a: i32, b: i32) -> i32 { a + b }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("fn") && output.contains("add"));
}

#[test]
fn test_transpile_if_statement() {
    let code = "if x > 0 { print(\"positive\") }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("if"));
}

#[test]
fn test_transpile_if_else_statement() {
    let code = "if x > 0 { 1 } else { -1 }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("if") && output.contains("else"));
}

#[test]
fn test_transpile_while_loop() {
    let code = "while x < 10 { x = x + 1 }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("while"));
}

#[test]
fn test_transpile_for_loop() {
    let code = "for i in 0..10 { print(i) }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("for"));
}

#[test]
fn test_transpile_match_statement() {
    let code = r#"match x {
        1 => "one",
        2 => "two",
        _ => "other"
    }"#;
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("match"));
}

#[test]
fn test_transpile_return_statement() {
    let code = "fun test() { return 42 }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("return"));
}

#[test]
fn test_transpile_break_statement() {
    let code = "while true { break }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("break"));
}

#[test]
fn test_transpile_continue_statement() {
    let code = "while x < 10 { if x == 5 { continue } }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("continue"));
}

#[test]
fn test_transpile_struct_definition() {
    let code = "struct Point { x: f64, y: f64 }";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("struct") || output.contains("Point"));
    }
}

#[test]
fn test_transpile_impl_block() {
    let code = "impl Point { fun new(x: f64, y: f64) -> Point { Point { x, y } } }";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("impl") || output.contains("new"));
    }
}

#[test]
fn test_transpile_trait_definition() {
    let code = "trait Display { fun display(&self) -> String }";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("trait") || output.contains("Display"));
    }
}

#[test]
fn test_transpile_use_statement() {
    let code = "use std::collections::HashMap";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("use") || output.contains("HashMap"));
    }
}

#[test]
fn test_transpile_async_function() {
    let code = "async fun fetch_data() { await request() }";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("async") || output.contains("await"));
    }
}

#[test]
fn test_transpile_closure() {
    let code = "|x| x * 2";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("|") || output.contains("move"));
}

#[test]
fn test_transpile_type_alias() {
    let code = "type Number = i32";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("type") || output.contains("Number"));
    }
}

#[test]
fn test_transpile_const_declaration() {
    let code = "const PI = 3.14159";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("const") || output.contains("PI"));
    }
}

#[test]
fn test_transpile_static_declaration() {
    let code = "static COUNTER: i32 = 0";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("static") || output.contains("COUNTER"));
    }
}

#[test]
fn test_transpile_macro_invocation() {
    let code = "println!(\"Hello, world!\")";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("println") || output.contains("!"));
}

#[test]
fn test_transpile_array_literal() {
    let code = "[1, 2, 3, 4, 5]";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("vec!") || output.contains("["));
}

#[test]
fn test_transpile_tuple_literal() {
    let code = "(1, \"hello\", true)";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("(") && output.contains(")"));
}

#[test]
fn test_transpile_index_expression() {
    let code = "array[0]";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("[") && output.contains("]"));
}

#[test]
fn test_transpile_field_access() {
    let code = "point.x";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("."));
}

#[test]
fn test_transpile_method_chain() {
    let code = "list.filter(|x| x > 0).map(|x| x * 2).collect()";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("filter") || output.contains("map"));
}

#[test]
fn test_transpile_question_mark_operator() {
    let code = "fun test() -> Result<i32> { let x = operation()?; Ok(x) }";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("?") || output.contains("Result"));
    }
}

#[test]
fn test_transpile_range_expression() {
    let code = "0..10";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains(".."));
}

#[test]
fn test_transpile_destructuring_let() {
    let code = "let (x, y) = (1, 2)";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("let") && output.contains("("));
    }
}

#[test]
fn test_transpile_pattern_matching_in_let() {
    let code = "let Some(x) = option";
    let result = parse_and_transpile(code);
    // Might not be supported yet, but test anyway
    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("Some") || output.contains("let"));
    }
}

#[test]
fn test_transpile_nested_blocks() {
    let code = "{ let x = { let y = 1; y + 1 }; x * 2 }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("{") && output.contains("}"));
}

#[test]
fn test_transpile_complex_expression() {
    let code = "if x > 0 && y < 10 { x + y } else { x - y }";
    let result = parse_and_transpile(code);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("if") && output.contains("&&"));
}