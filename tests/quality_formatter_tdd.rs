//! TDD tests for quality formatter functionality
//! Target: Improve coverage from 0% to 80%+

use ruchy::quality::Formatter;
use ruchy::Parser;

#[test]
fn test_format_simple_expression() {
    let formatter = Formatter::new();
    
    let input = "let   x=5";
    let expected = "let x = 5";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_function_definition() {
    let formatter = Formatter::new();
    
    let input = "fun   add(x,y){x+y}";
    let expected = "fun add(x, y) {\n    x + y\n}";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_if_else() {
    let formatter = Formatter::new();
    
    let input = "if x>5{println(\"big\")}else{println(\"small\")}";
    let expected = "if x > 5 {\n    println(\"big\")\n} else {\n    println(\"small\")\n}";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_list_literal() {
    let formatter = Formatter::new();
    
    let input = "[1,2,3,4,5]";
    let expected = "[1, 2, 3, 4, 5]";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_object_literal() {
    let formatter = Formatter::new();
    
    let input = "{name:\"Alice\",age:30}";
    let expected = "{\n    name: \"Alice\",\n    age: 30\n}";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_match_expression() {
    let formatter = Formatter::new();
    
    let input = "match x{1=>\"one\",2=>\"two\",_=>\"other\"}";
    let expected = "match x {\n    1 => \"one\",\n    2 => \"two\",\n    _ => \"other\"\n}";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_loop_constructs() {
    let formatter = Formatter::new();
    
    let input = "for i in 1..10{println(i)}";
    let expected = "for i in 1..10 {\n    println(i)\n}";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
    
    let input = "while x<10{x=x+1}";
    let expected = "while x < 10 {\n    x = x + 1\n}";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_nested_blocks() {
    let formatter = Formatter::new();
    
    let input = "fun outer(){fun inner(){42}inner()}";
    let expected = "fun outer() {\n    fun inner() {\n        42\n    }\n    inner()\n}";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_comments() {
    let formatter = Formatter::new();
    
    let input = "// This is a comment\nlet x = 5 // inline comment";
    let expected = "// This is a comment\nlet x = 5 // inline comment";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_multiline_strings() {
    let formatter = Formatter::new();
    
    let input = "let msg = \"multi\nline\nstring\"";
    let expected = "let msg = \"multi\nline\nstring\"";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_pipeline_operator() {
    let formatter = Formatter::new();
    
    let input = "[1,2,3]|>map(x=>x*2)|>filter(x=>x>2)";
    let expected = "[1, 2, 3]\n    |> map(x => x * 2)\n    |> filter(x => x > 2)";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_lambda_expressions() {
    let formatter = Formatter::new();
    
    let input = "let double=x=>x*2";
    let expected = "let double = x => x * 2";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
    
    let input = "let add=(x,y)=>x+y";
    let expected = "let add = (x, y) => x + y";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result.trim(), expected);
}

#[test]
fn test_format_preserves_semantics() {
    let formatter = Formatter::new();
    
    // Format shouldn't change the meaning
    let input = "let x=5+3*2";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let result = formatter.format(&ast).unwrap();
    assert_eq!(result, "let x = 5 + 3 * 2"); // Preserves precedence
}

#[test]
fn test_format_idempotent() {
    let formatter = Formatter::new();
    
    let input = "let x = 5";
    let once = formatter.format(input).unwrap();
    let mut parser2 = Parser::new(&once);
    let ast2 = parser2.parse().unwrap();
    let twice = formatter.format(&ast2).unwrap();
    assert_eq!(once, twice); // Formatting twice gives same result
}