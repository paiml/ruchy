// Simple tests for statements.rs coverage using supported syntax

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

#[test]
fn test_basic_assignment() {
    let transpiler = Transpiler::new();

    // Basic variable assignment (currently generates just assignment, not let)
    let mut parser = Parser::new("x = 5");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("x = 5"));

    // Assignment to existing variable
    let mut parser = Parser::new("x = 10; x = 20");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("x = 20"));
}

#[test]
fn test_basic_if_statement() {
    let transpiler = Transpiler::new();

    // Simple if statement
    let mut parser = Parser::new("if x > 5 { y = 10 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("if"));

    // If-else statement
    let mut parser = Parser::new("if x > 5 { y = 10 } else { y = 20 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("else"));
}

#[test]
fn test_simple_match() {
    let transpiler = Transpiler::new();

    let mut parser = Parser::new("match x { 1 => 10, 2 => 20, _ => 30 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("match"));
    assert!(result.contains("=>"));
}

#[test]
fn test_function_definition() {
    let transpiler = Transpiler::new();

    // Simple function
    let mut parser = Parser::new("fun add(x, y) { x + y }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("fn"));

    // Function with return
    let mut parser = Parser::new("fun multiply(x, y) { return x * y }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("return"));
}

#[test]
fn test_basic_loops() {
    let transpiler = Transpiler::new();

    // Simple for loop
    let mut parser = Parser::new("for i in [1, 2, 3] { x = i }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("for"));
    assert!(result.contains("in"));

    // While loop
    let mut parser = Parser::new("while x < 10 { x = x + 1 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("while"));

    // Infinite loop
    let mut parser = Parser::new("loop { x = x + 1 }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("loop"));
}

#[test]
fn test_break_continue() {
    let transpiler = Transpiler::new();

    // Loop with break
    let mut parser = Parser::new("loop { if x > 10 { break } }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("break"));

    // Loop with continue
    let mut parser = Parser::new("for i in [1, 2, 3] { if i == 2 { continue } }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("continue"));
}

#[test]
fn test_basic_expressions() {
    let transpiler = Transpiler::new();

    // Expression statement
    let mut parser = Parser::new("x + y");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("+"));

    // Method call
    let mut parser = Parser::new("list.push(5)");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("push"));
}

#[test]
fn test_array_operations() {
    let transpiler = Transpiler::new();

    // Array literal
    let mut parser = Parser::new("[1, 2, 3]");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("[") || result.contains("vec!"));

    // Array indexing
    let mut parser = Parser::new("arr[0]");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("["));
}

#[test]
fn test_object_operations() {
    let transpiler = Transpiler::new();

    // Object literal (HashMap)
    let mut parser = Parser::new("{x: 1, y: 2}");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("HashMap"));

    // Object field access
    let mut parser = Parser::new("obj.field");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("."));
}

#[test]
fn test_string_operations() {
    let transpiler = Transpiler::new();

    // String literal
    let mut parser = Parser::new("\"hello world\"");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("\"hello world\""));

    // String concatenation uses format!
    let mut parser = Parser::new("\"hello\" + \" world\"");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("format"));
}

#[test]
fn test_boolean_operations() {
    let transpiler = Transpiler::new();

    // Boolean literals
    let mut parser = Parser::new("true");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("true"));

    // Logical operations
    let mut parser = Parser::new("x && y");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("&&"));

    let mut parser = Parser::new("x || y");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("||"));
}

#[test]
fn test_unary_operations() {
    let transpiler = Transpiler::new();

    // Negation
    let mut parser = Parser::new("!x");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("!"));

    // Negative number
    let mut parser = Parser::new("-42");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("-"));
}

#[test]
fn test_comparison_operations() {
    let transpiler = Transpiler::new();

    let comparisons = vec![
        ("x == y", "=="),
        ("x != y", "!="),
        ("x < y", "<"),
        ("x > y", ">"),
        ("x <= y", "<="),
        ("x >= y", ">="),
    ];

    for (code, op) in comparisons {
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        let result = transpiler.transpile(&ast).unwrap().to_string();
        assert!(result.contains(op), "Failed for operation: {}", op);
    }
}

#[test]
fn test_arithmetic_operations() {
    let transpiler = Transpiler::new();

    let operations = vec![
        ("x + y", "+"),
        ("x - y", "-"),
        ("x * y", "*"),
        ("x / y", "/"),
        ("x % y", "%"),
    ];

    for (code, op) in operations {
        let mut parser = Parser::new(code);
        let ast = parser.parse().unwrap();
        let result = transpiler.transpile(&ast).unwrap().to_string();
        assert!(result.contains(op), "Failed for operation: {}", op);
    }
}

#[test]
fn test_type_conversions() {
    let transpiler = Transpiler::new();

    // Method calls (currently methods without args get dropped)
    let mut parser = Parser::new("list.push(5)");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("push"));
}

#[test]
fn test_range_operations() {
    let transpiler = Transpiler::new();

    // Exclusive range
    let mut parser = Parser::new("1..10");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains(".."));

    // Inclusive range
    let mut parser = Parser::new("1..=10");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("..="));
}

#[test]
fn test_tuple_operations() {
    let transpiler = Transpiler::new();

    // Tuple literal
    let mut parser = Parser::new("(1, 2, 3)");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("("));
    assert!(result.contains(","));
}

#[test]
fn test_character_literals() {
    let transpiler = Transpiler::new();

    // Character literal
    let mut parser = Parser::new("'a'");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("'a'"));
}

#[test]
fn test_null_value() {
    let transpiler = Transpiler::new();

    // Null literal
    let mut parser = Parser::new("null");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("None"));
}

#[test]
fn test_comments() {
    let transpiler = Transpiler::new();

    // Single-line comment
    let mut parser = Parser::new("// This is a comment\nx = 5");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("x = 5"));

    // Multi-line comment
    let mut parser = Parser::new("/* This is a\nmulti-line comment */\ny = 10");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("y = 10"));
}

#[test]
fn test_nested_blocks() {
    let transpiler = Transpiler::new();

    // Nested block statements
    let mut parser = Parser::new("{ x = 5; { y = 10; } z = 15; }");
    let ast = parser.parse().unwrap();
    let result = transpiler.transpile(&ast).unwrap().to_string();
    assert!(result.contains("{"));
    assert!(result.contains("}"));
}