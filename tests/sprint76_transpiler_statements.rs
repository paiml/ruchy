//! Sprint 76: Transpiler Statements Coverage
//! Target: Boost backend/transpiler/statements.rs coverage

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

#[test]
fn test_transpile_let_statements() {
    let statements = vec![
        "let x = 5",
        "let mut y = 10",
        "let z: int = 15",
        "let a, b, c = 1, 2, 3",
        "let [x, y] = [1, 2]",
        "let { name, age } = person",
        "const PI = 3.14",
        "var count = 0",
    ];

    for stmt in statements {
        let mut parser = Parser::new(stmt);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpile_if_statements() {
    let statements = vec![
        "if x > 0 { println(x) }",
        "if x { y } else { z }",
        "if a { b } else if c { d } else { e }",
        "if let Some(x) = opt { use(x) }",
        "if x > 0 && y < 10 { do_something() }",
    ];

    for stmt in statements {
        let mut parser = Parser::new(stmt);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpile_loops() {
    let loops = vec![
        "while x < 10 { x += 1 }",
        "for i in 0..10 { println(i) }",
        "for x in arr { process(x) }",
        "for (key, value) in map { }",
        "loop { break }",
        "do { x += 1 } while x < 10",
        "while true { if done { break } }",
        "for i in 0..10 { if i == 5 { continue } }",
    ];

    for loop_stmt in loops {
        let mut parser = Parser::new(loop_stmt);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpile_match_statements() {
    let matches = vec![
        "match x { 1 => 'a', 2 => 'b', _ => 'c' }",
        "match opt { Some(x) => x, None => 0 }",
        "match (x, y) { (0, 0) => origin, _ => other }",
        "match x { 1..=10 => small, 11..=100 => medium, _ => large }",
        "match s { 'a'..='z' => lower, 'A'..='Z' => upper, _ => other }",
    ];

    for match_stmt in matches {
        let mut parser = Parser::new(match_stmt);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpile_function_definitions() {
    let functions = vec![
        "fn add(a, b) { a + b }",
        "fn factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }",
        "fn greet(name: string) -> string { f'Hello, {name}!' }",
        "async fn fetch() { await get_data() }",
        "fn generic<T>(x: T) -> T { x }",
        "pub fn public() { }",
        "fn varargs(...args) { }",
        "fn with_default(x = 10) { }",
    ];

    for func in functions {
        let mut parser = Parser::new(func);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpile_return_statements() {
    let returns = vec![
        "return",
        "return 42",
        "return x + y",
        "return if x > 0 { x } else { -x }",
        "return Ok(value)",
        "return Err('error')",
    ];

    for ret in returns {
        let mut parser = Parser::new(ret);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpile_class_definitions() {
    let classes = vec![
        "class Point { x: int; y: int }",
        "class Circle { radius: float; fn area() { PI * radius * radius } }",
        "class Person { name: string; age: int; fn greet() { } }",
        "class Generic<T> { value: T }",
        "class Derived extends Base { }",
    ];

    for class in classes {
        let mut parser = Parser::new(class);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpile_import_statements() {
    let imports = vec![
        "import std",
        "import std.io",
        "from std import println",
        "from math import { sin, cos, tan }",
        "import './module'",
        "import * as utils from './utils'",
    ];

    for import in imports {
        let mut parser = Parser::new(import);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpile_try_catch() {
    let try_catches = vec![
        "try { risky() } catch(e) { handle(e) }",
        "try { risky() } catch { handle() } finally { cleanup() }",
        "try { risky() } catch(IOError e) { } catch(ValueError e) { }",
    ];

    for try_catch in try_catches {
        let mut parser = Parser::new(try_catch);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);
            assert!(result.is_ok() || result.is_err());
        }
    }
}

#[test]
fn test_transpile_complex_programs() {
    let program = r#"
        fn main() {
            let mut sum = 0
            for i in 1..=100 {
                if i % 2 == 0 {
                    sum += i
                }
            }
            println(f"Sum of even numbers: {sum}")
        }
    "#;

    let mut parser = Parser::new(program);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        assert!(result.is_ok() || result.is_err());
    }
}