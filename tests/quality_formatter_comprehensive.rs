//! Comprehensive test suite for the Quality Formatter module
//! Aims to improve code coverage from 0% to significant coverage

use ruchy::quality::formatter::Formatter;
use ruchy::frontend::parser::Parser;

#[test]
fn test_formatter_new() {
    let formatter = Formatter::new();
    // Formatter should be created successfully
    let code = "42";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert_eq!(formatted, "42");
}

#[test]
fn test_format_integer_literal() {
    let formatter = Formatter::new();
    let code = "123";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert_eq!(formatted, "123");
}

#[test]
fn test_format_float_literal() {
    let formatter = Formatter::new();
    let code = "3.14";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("3.14"));
}

#[test]
fn test_format_string_literal() {
    let formatter = Formatter::new();
    let code = "\"hello world\"";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert_eq!(formatted, "\"hello world\"");
}

#[test]
fn test_format_boolean_literals() {
    let formatter = Formatter::new();
    
    let code = "true";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert_eq!(formatted, "true");
    
    let code = "false";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert_eq!(formatted, "false");
}

#[test]
fn test_format_identifier() {
    let formatter = Formatter::new();
    let code = "variable_name";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert_eq!(formatted, "variable_name");
}

#[test]
fn test_format_binary_operation() {
    let formatter = Formatter::new();
    let code = "1 + 2";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("1"));
    assert!(formatted.contains("+"));
    assert!(formatted.contains("2"));
}

#[test]
fn test_format_unary_operation() {
    let formatter = Formatter::new();
    let code = "-5";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("-"));
    assert!(formatted.contains("5"));
}

#[test]
fn test_format_list_literal() {
    let formatter = Formatter::new();
    let code = "[1, 2, 3]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("["));
    assert!(formatted.contains("1"));
    assert!(formatted.contains("2"));
    assert!(formatted.contains("3"));
    assert!(formatted.contains("]"));
}

#[test]
fn test_format_tuple_literal() {
    let formatter = Formatter::new();
    let code = "(1, 2, 3)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("("));
    assert!(formatted.contains("1"));
    assert!(formatted.contains("2"));
    assert!(formatted.contains("3"));
    assert!(formatted.contains(")"));
}

#[test]
fn test_format_function_call() {
    let formatter = Formatter::new();
    let code = "print(\"hello\")";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("print"));
    assert!(formatted.contains("("));
    assert!(formatted.contains("\"hello\""));
    assert!(formatted.contains(")"));
}

#[test]
fn test_format_lambda() {
    let formatter = Formatter::new();
    let code = "|x| x + 1";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("|"));
    assert!(formatted.contains("x"));
    assert!(formatted.contains("+"));
    assert!(formatted.contains("1"));
}

#[test]
fn test_format_if_expression() {
    let formatter = Formatter::new();
    let code = "if true { 1 } else { 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("if"));
    assert!(formatted.contains("true"));
    assert!(formatted.contains("1"));
    assert!(formatted.contains("else"));
    assert!(formatted.contains("2"));
}

#[test]
fn test_format_let_binding() {
    let formatter = Formatter::new();
    let code = "let x = 5";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("let"));
    assert!(formatted.contains("x"));
    assert!(formatted.contains("="));
    assert!(formatted.contains("5"));
}

#[test]
fn test_format_function_definition() {
    let formatter = Formatter::new();
    let code = "fun add(x, y) { x + y }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("fun"));
    assert!(formatted.contains("add"));
    assert!(formatted.contains("x"));
    assert!(formatted.contains("y"));
}

#[test]
fn test_format_match_expression() {
    let formatter = Formatter::new();
    let code = "match x { 1 => \"one\", 2 => \"two\", _ => \"other\" }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("match"));
    assert!(formatted.contains("x"));
    assert!(formatted.contains("1"));
    assert!(formatted.contains("=>"));
    assert!(formatted.contains("\"one\""));
}

#[test]
fn test_format_for_loop() {
    let formatter = Formatter::new();
    let code = "for i in 1..10 { print(i) }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("for"));
    assert!(formatted.contains("i"));
    assert!(formatted.contains("in"));
}

#[test]
fn test_format_while_loop() {
    let formatter = Formatter::new();
    let code = "while x < 10 { x = x + 1 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("while"));
    assert!(formatted.contains("x"));
    assert!(formatted.contains("<"));
    assert!(formatted.contains("10"));
}

#[test]
fn test_format_block_expression() {
    let formatter = Formatter::new();
    let code = "{ let x = 1; x + 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("{"));
    assert!(formatted.contains("let"));
    assert!(formatted.contains("x"));
    assert!(formatted.contains("}"));
}

#[test]
fn test_format_index_expression() {
    let formatter = Formatter::new();
    let code = "array[0]";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("array"));
    assert!(formatted.contains("["));
    assert!(formatted.contains("0"));
    assert!(formatted.contains("]"));
}

#[test]
fn test_format_field_access() {
    let formatter = Formatter::new();
    let code = "obj.field";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("obj"));
    assert!(formatted.contains("."));
    assert!(formatted.contains("field"));
}

#[test]
fn test_format_method_call() {
    let formatter = Formatter::new();
    let code = "obj.method()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("obj"));
    assert!(formatted.contains("."));
    assert!(formatted.contains("method"));
    assert!(formatted.contains("()"));
}

#[test]
fn test_format_async_function() {
    let formatter = Formatter::new();
    let code = "async fun fetch() { 42 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("async"));
    assert!(formatted.contains("fun"));
    assert!(formatted.contains("fetch"));
}

#[test]
fn test_format_await_expression() {
    let formatter = Formatter::new();
    let code = "await fetch()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("await"));
    assert!(formatted.contains("fetch"));
}

#[test]
fn test_format_range_expression() {
    let formatter = Formatter::new();
    let code = "1..10";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("1"));
    assert!(formatted.contains(".."));
    assert!(formatted.contains("10"));
}

#[test]
fn test_format_struct_literal() {
    let formatter = Formatter::new();
    let code = "Point { x: 1, y: 2 }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("Point"));
    assert!(formatted.contains("x"));
    assert!(formatted.contains("1"));
    assert!(formatted.contains("y"));
    assert!(formatted.contains("2"));
}

#[test]
fn test_format_pipeline_operator() {
    let formatter = Formatter::new();
    let code = "5 |> double |> increment";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("5"));
    assert!(formatted.contains("|>"));
    assert!(formatted.contains("double"));
    assert!(formatted.contains("increment"));
}

#[test]
fn test_format_char_literal() {
    let formatter = Formatter::new();
    let code = "'a'";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert_eq!(formatted, "'a'");
}

#[test]
fn test_format_unit_literal() {
    let formatter = Formatter::new();
    let code = "()";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert_eq!(formatted, "()");
}

#[test]
fn test_format_complex_expression() {
    let formatter = Formatter::new();
    let code = "(1 + 2) * (3 - 4)";
    let mut parser = Parser::new(code);
    let ast = parser.parse().unwrap();
    let formatted = formatter.format(&ast).unwrap();
    assert!(formatted.contains("1"));
    assert!(formatted.contains("+"));
    assert!(formatted.contains("2"));
    assert!(formatted.contains("*"));
    assert!(formatted.contains("3"));
    assert!(formatted.contains("-"));
    assert!(formatted.contains("4"));
}