// EXTREME Coverage Test Suite for src/frontend/parser/core.rs
// Target: Maximum coverage for Parser
// Sprint 80: ALL NIGHT Coverage Marathon
//
// Quality Standards:
// - Exhaustive testing
// - Zero uncovered lines

use ruchy::Parser;

// Basic functionality
#[test]
fn test_parser_new() {
    let parser = Parser::new("42");
    assert_eq!(parser.get_errors().len(), 0);
}

#[test]
fn test_parse_integer() {
    let mut parser = Parser::new("42");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_string() {
    let mut parser = Parser::new("\"hello\"");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_boolean_true() {
    let mut parser = Parser::new("true");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_boolean_false() {
    let mut parser = Parser::new("false");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_identifier() {
    let mut parser = Parser::new("variable_name");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_arithmetic() {
    let mut parser = Parser::new("1 + 2 * 3");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_parentheses() {
    let mut parser = Parser::new("(1 + 2) * 3");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_let_binding() {
    let mut parser = Parser::new("let x = 42 in x + 1");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_function() {
    let mut parser = Parser::new("fn(x) => x * 2");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_if_expression() {
    let mut parser = Parser::new("if true { 1 } else { 2 }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_match_expression() {
    let mut parser = Parser::new("match x { 1 => \"one\", _ => \"other\" }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_list() {
    let mut parser = Parser::new("[1, 2, 3]");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_tuple() {
    let mut parser = Parser::new("(1, \"hello\", true)");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_block() {
    let mut parser = Parser::new("{ let x = 1; let y = 2; x + y }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_multiple_statements() {
    let mut parser = Parser::new("let x = 1; let y = 2; x + y");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_empty() {
    let mut parser = Parser::new("");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_parse_expr_method() {
    let mut parser = Parser::new("42 + 8");
    let result = parser.parse_expr();
    assert!(result.is_ok());
}

#[test]
fn test_parse_complex_expression() {
    let mut parser = Parser::new("fn(x, y) => if x > y { x } else { y }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_nested_functions() {
    let mut parser = Parser::new("fn(x) => fn(y) => x + y");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_method_call() {
    let mut parser = Parser::new("object.method(arg1, arg2)");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_field_access() {
    let mut parser = Parser::new("object.field");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_index_access() {
    let mut parser = Parser::new("array[0]");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_unary_operators() {
    let mut parser = Parser::new("-42");
    let result = parser.parse();
    assert!(result.is_ok());

    let mut parser = Parser::new("!true");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_comparison() {
    let mut parser = Parser::new("x > 5 && y < 10");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_for_loop() {
    let mut parser = Parser::new("for i in [1, 2, 3] { print(i) }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_while_loop() {
    let mut parser = Parser::new("while x < 10 { x = x + 1 }");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_async_function() {
    let mut parser = Parser::new("async fn() => await fetch()");
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_string_interpolation() {
    let mut parser = Parser::new("f\"Hello {name}\"");
    let result = parser.parse();
    assert!(result.is_ok());
}

// Error recovery
#[test]
fn test_parse_with_errors() {
    let mut parser = Parser::new("((((");
    let result = parser.parse();
    assert!(result.is_err());
}

#[test]
fn test_get_errors_empty() {
    let parser = Parser::new("42");
    assert!(parser.get_errors().is_empty());
}

#[test]
fn test_parse_semicolon_separated() {
    let mut parser = Parser::new("1; 2; 3");
    let result = parser.parse();
    assert!(result.is_ok());
}

// Stress tests
#[test]
fn test_parse_many_expressions() {
    for i in 0..100 {
        let input = i.to_string();
        let mut parser = Parser::new(&input);
        let _ = parser.parse();
    }
}

#[test]
fn test_parse_deep_nesting() {
    let expr = "((((((((((42))))))))))";
    let mut parser = Parser::new(expr);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
fn test_parse_long_expression() {
    let expr = "1 + 2 + 3 + 4 + 5 + 6 + 7 + 8 + 9 + 10";
    let mut parser = Parser::new(expr);
    let result = parser.parse();
    assert!(result.is_ok());
}