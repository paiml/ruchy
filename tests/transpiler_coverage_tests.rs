#![allow(clippy::unwrap_used, clippy::panic)]
//! Additional transpiler tests to improve coverage

use anyhow::Result;
use ruchy::{Parser, Transpiler};

#[test]
fn test_transpile_literals() -> Result<()> {
    let cases = vec![
        ("42", "42"),
        ("3.14", "3.14"),
        ("true", "true"),
        ("false", "false"),
        ("\"hello\"", "\"hello\""),
    ];

    for (input, expected_contains) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast)?;
        let code_str = rust_code.to_string();
        assert!(
            code_str.contains(expected_contains),
            "Failed for input: {input}, got: {code_str}"
        );
    }

    Ok(())
}

#[test]
fn test_transpile_let_binding() -> Result<()> {
    let input = "let x = 42";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("let x"));
    assert!(code_str.contains("42"));

    Ok(())
}

#[test]
fn test_transpile_function() -> Result<()> {
    let input = "fun add(a: i32, b: i32) -> i32 { a + b }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("fn add"));
    assert!(code_str.contains("i32"));

    Ok(())
}

#[test]
fn test_transpile_if_expression() -> Result<()> {
    let input = "if x > 0 { positive } else { negative }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("if"));
    assert!(code_str.contains("else"));

    Ok(())
}

#[test]
fn test_transpile_match_expression() -> Result<()> {
    let input = r#"
        match x {
            0 => "zero",
            _ => "other"
        }
    "#;

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("match"));
    assert!(code_str.contains("=>"));

    Ok(())
}

#[test]
fn test_transpile_struct() -> Result<()> {
    let input = "struct Point { x: f64, y: f64 }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("struct Point"));
    assert!(code_str.contains("f64"));

    Ok(())
}

#[test]
fn test_transpile_impl_block() -> Result<()> {
    let input = r"
        impl Point {
            fun new(x: f64, y: f64) -> Point {
                Point { x: x, y: y }
            }
        }
    ";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("impl Point"));
    assert!(code_str.contains("fn new"));

    Ok(())
}

#[test]
fn test_transpile_trait() -> Result<()> {
    let input = r"
        trait Display {
            fun fmt(&self) -> String
        }
    ";

    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("trait Display"));
    assert!(code_str.contains("fn fmt"));

    Ok(())
}

#[test]
fn test_transpile_for_loop() -> Result<()> {
    let input = "for x in list { print(x) }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("for x in"));

    Ok(())
}

#[test]
fn test_transpile_while_loop() -> Result<()> {
    let input = "while x < 10 { x = x + 1 }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("while"));

    Ok(())
}

#[test]
fn test_transpile_list_literal() -> Result<()> {
    let input = "[1, 2, 3]";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("vec!"));

    Ok(())
}

#[test]
fn test_transpile_lambda() -> Result<()> {
    let input = "fun (x) { x * 2 }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains('|'));

    Ok(())
}

#[test]
fn test_transpile_method_call() -> Result<()> {
    let input = "obj.method(arg)";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains(".method"));

    Ok(())
}

#[test]
fn test_transpile_binary_ops() -> Result<()> {
    let cases = vec![
        ("a + b", "+"),
        ("a - b", "-"),
        ("a * b", "*"),
        ("a / b", "/"),
        ("a % b", "%"),
        ("a == b", "=="),
        ("a != b", "!="),
        ("a < b", "<"),
        ("a <= b", "<="),
        ("a > b", ">"),
        ("a >= b", ">="),
        ("a && b", "&&"),
        ("a || b", "||"),
    ];

    for (input, expected_op) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast)?;
        let code_str = rust_code.to_string();
        assert!(
            code_str.contains(expected_op),
            "Failed for input: {input}, got: {code_str}"
        );
    }

    Ok(())
}

#[test]
fn test_transpile_unary_ops() -> Result<()> {
    let cases = vec![("-x", "-"), ("!x", "!")];

    for (input, expected_op) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse()?;
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast)?;
        let code_str = rust_code.to_string();
        assert!(
            code_str.contains(expected_op),
            "Failed for input: {input}, got: {code_str}"
        );
    }

    Ok(())
}

#[test]
fn test_transpile_block() -> Result<()> {
    let input = "{ let x = 1; let y = 2; x + y }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains('{'));
    assert!(code_str.contains('}'));

    Ok(())
}

#[test]
fn test_transpile_call_expression() -> Result<()> {
    let input = "func(arg1, arg2)";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("func"));
    assert!(code_str.contains("arg1"));
    assert!(code_str.contains("arg2"));

    Ok(())
}

#[test]
fn test_transpile_generic_function() -> Result<()> {
    let input = "fun identity<T>(x: T) -> T { x }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("fn identity"));
    assert!(code_str.contains("<T>"));

    Ok(())
}

#[test]
fn test_transpile_generic_struct() -> Result<()> {
    let input = "struct Container<T> { value: T }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("struct Container"));
    assert!(code_str.contains("<T>"));

    Ok(())
}

#[test]
fn test_transpile_struct_literal() -> Result<()> {
    let input = "Point { x: 10, y: 20 }";
    let mut parser = Parser::new(input);
    let ast = parser.parse()?;
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast)?;
    let code_str = rust_code.to_string();

    assert!(code_str.contains("Point"));
    assert!(code_str.contains("x :"));
    assert!(code_str.contains("y :"));

    Ok(())
}
