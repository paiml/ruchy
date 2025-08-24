#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::unwrap_used)]
#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::uninlined_format_args
)]

use insta::{assert_debug_snapshot, assert_snapshot};
use ruchy::{Parser, Transpiler};

#[test]
fn snapshot_parse_let_statement() {
    let input = "let x = 42";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    assert_debug_snapshot!(ast);
}

#[test]
fn snapshot_parse_function() {
    let input = r"
        fn fibonacci(n: i32) -> i32 {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
    ";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    assert_debug_snapshot!(ast);
}

#[test]
fn snapshot_parse_match_expression() {
    let input = r#"
        match x {
            0 => "zero",
            1 => "one",
            _ => "many"
        }
    "#;
    let mut parser = Parser::new(input);
    let expr = parser.parse_expr().unwrap();
    assert_debug_snapshot!(expr);
}

#[test]
fn snapshot_transpile_println() {
    let input = r#"println("Hello, World!")"#;
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    assert_snapshot!(output);
}

#[test]
fn snapshot_transpile_function() {
    let input = r"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
    ";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    assert_snapshot!(output);
}

#[test]
fn snapshot_transpile_struct() {
    let input = r"
        struct Point {
            x: f64,
            y: f64
        }
    ";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    assert_snapshot!(output);
}

#[test]
fn snapshot_transpile_match() {
    let input = r#"
        fn describe_number(n: i32) -> String {
            match n {
                0 => "zero",
                1 => "one",
                2..=10 => "small",
                _ => "large"
            }
        }
    "#;
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    assert_snapshot!(output);
}

#[test]
fn snapshot_transpile_pipeline() {
    let input = r"
        let result = data
            >> filter(|x| x > 0)
            >> map(|x| x * 2)
            >> sum()
    ";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    assert_snapshot!(output);
}

#[test]
fn snapshot_transpile_async_actor() {
    let input = r"
        actor Counter {
            state count: i32 = 0
            
            handle increment() {
                self.count += 1
            }
            
            handle get() -> i32 {
                self.count
            }
        }
    ";
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    assert_snapshot!(output);
}

#[test]
fn snapshot_transpile_result_type() {
    let input = r#"
        fn divide(a: f64, b: f64) -> Result<f64, String> {
            if b == 0.0 {
                Err("Division by zero")
            } else {
                Ok(a / b)
            }
        }
    "#;
    let mut parser = Parser::new(input);
    let ast = parser.parse().unwrap();
    let transpiler = Transpiler::new();
    let output = transpiler.transpile(&ast).unwrap();
    assert_snapshot!(output);
}
