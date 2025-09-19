// SNAPSHOT REGRESSION TESTS - Prevent Output Changes
// Target: Ensure consistent compilation output
// Sprint 80: ALL NIGHT Coverage Marathon Phase 21

use insta::assert_snapshot;
use ruchy::{Parser, Transpiler};

// Snapshot tests for parser output
#[test]
fn snapshot_parse_literals() {
    let cases = vec![
        ("42", "integer_literal"),
        ("3.14", "float_literal"),
        (r#""hello""#, "string_literal"),
        ("true", "bool_true"),
        ("false", "bool_false"),
    ];

    for (input, name) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        assert_snapshot!(name, format!("{:?}", ast));
    }
}

#[test]
fn snapshot_parse_arithmetic() {
    let cases = vec![
        ("1 + 2", "addition"),
        ("5 - 3", "subtraction"),
        ("2 * 3", "multiplication"),
        ("10 / 2", "division"),
        ("7 % 3", "modulo"),
        ("2 ** 8", "exponent"),
    ];

    for (input, name) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        assert_snapshot!(name, format!("{:?}", ast));
    }
}

#[test]
fn snapshot_parse_comparison() {
    let cases = vec![
        ("a == b", "equality"),
        ("a != b", "inequality"),
        ("a < b", "less_than"),
        ("a > b", "greater_than"),
        ("a <= b", "less_equal"),
        ("a >= b", "greater_equal"),
    ];

    for (input, name) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        assert_snapshot!(name, format!("{:?}", ast));
    }
}

#[test]
fn snapshot_parse_logical() {
    let cases = vec![
        ("a && b", "logical_and"),
        ("a || b", "logical_or"),
        ("!a", "logical_not"),
    ];

    for (input, name) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        assert_snapshot!(name, format!("{:?}", ast));
    }
}

#[test]
fn snapshot_parse_control_flow() {
    let cases = vec![
        ("if x { 1 } else { 2 }", "if_else"),
        ("match x { 1 => a, _ => b }", "match_expr"),
        ("while x { y }", "while_loop"),
        ("for i in list { x }", "for_loop"),
    ];

    for (input, name) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        assert_snapshot!(name, format!("{:?}", ast));
    }
}

#[test]
fn snapshot_parse_functions() {
    let cases = vec![
        ("fn f() { 42 }", "simple_function"),
        ("fn add(x, y) { x + y }", "function_with_params"),
        ("fn() => 42", "lambda_no_params"),
        ("fn(x) => x * 2", "lambda_with_param"),
    ];

    for (input, name) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        assert_snapshot!(name, format!("{:?}", ast));
    }
}

#[test]
fn snapshot_parse_data_structures() {
    let cases = vec![
        ("[1, 2, 3]", "list"),
        ("(1, 2, 3)", "tuple"),
        ("{a: 1, b: 2}", "object"),
    ];

    for (input, name) in cases {
        let mut parser = Parser::new(input);
        let ast = parser.parse();
        assert_snapshot!(name, format!("{:?}", ast));
    }
}

// Snapshot tests for transpiler output
#[test]
fn snapshot_transpile_hello_world() {
    let source = r#"println("Hello, World!")"#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();

    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast);

    assert_snapshot!("hello_world_transpiled", rust_code);
}

#[test]
fn snapshot_transpile_factorial() {
    let source = r#"
        fn factorial(n) {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    "#;

    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();

    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast);

    assert_snapshot!("factorial_transpiled", rust_code);
}

#[test]
fn snapshot_transpile_fibonacci() {
    let source = r#"
        fn fib(n) {
            if n <= 1 {
                n
            } else {
                fib(n - 1) + fib(n - 2)
            }
        }
    "#;

    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();

    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast);

    assert_snapshot!("fibonacci_transpiled", rust_code);
}

#[test]
fn snapshot_transpile_pattern_matching() {
    let source = r#"
        match value {
            0 => "zero",
            1 => "one",
            2..=10 => "small",
            _ => "large"
        }
    "#;

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("pattern_matching_transpiled", rust_code);
    }
}

#[test]
fn snapshot_transpile_async_await() {
    let source = r#"
        async fn fetch_data() {
            await fetch_api()
        }
    "#;

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("async_await_transpiled", rust_code);
    }
}

#[test]
fn snapshot_transpile_string_interpolation() {
    let source = r#"f"Hello {name}, you are {age} years old""#;

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("string_interpolation_transpiled", rust_code);
    }
}

#[test]
fn snapshot_transpile_list_comprehension() {
    let source = "[x * 2 for x in range(10) if x % 2 == 0]";

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("list_comprehension_transpiled", rust_code);
    }
}

#[test]
fn snapshot_transpile_closures() {
    let source = r#"
        let add = fn(x) {
            fn(y) { x + y }
        }
        let add5 = add(5)
        add5(3)
    "#;

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("closures_transpiled", rust_code);
    }
}

#[test]
fn snapshot_transpile_error_handling() {
    let source = r#"
        fn safe_divide(a, b) {
            if b == 0 {
                Err("Division by zero")
            } else {
                Ok(a / b)
            }
        }
    "#;

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("error_handling_transpiled", rust_code);
    }
}

#[test]
fn snapshot_transpile_generics() {
    let source = r#"
        fn identity<T>(x: T) -> T {
            x
        }
    "#;

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("generics_transpiled", rust_code);
    }
}

#[test]
fn snapshot_transpile_traits() {
    let source = r#"
        trait Display {
            fn display(&self) -> String
        }

        impl Display for Point {
            fn display(&self) -> String {
                f"({self.x}, {self.y})"
            }
        }
    "#;

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("traits_transpiled", rust_code);
    }
}

#[test]
fn snapshot_transpile_modules() {
    let source = r#"
        mod math {
            pub fn add(a, b) { a + b }
            pub fn multiply(a, b) { a * b }
        }

        use math::add
        add(1, 2)
    "#;

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("modules_transpiled", rust_code);
    }
}

// Complex program snapshots
#[test]
fn snapshot_transpile_complex_program() {
    let source = r#"
        // A complete program
        use std::collections::HashMap

        struct User {
            id: i32,
            name: String,
            email: String,
        }

        impl User {
            fn new(id: i32, name: String, email: String) -> User {
                User { id, name, email }
            }

            fn greet(&self) -> String {
                f"Hello, {self.name}!"
            }
        }

        fn main() {
            let users = vec![
                User::new(1, "Alice", "alice@example.com"),
                User::new(2, "Bob", "bob@example.com"),
            ]

            for user in users {
                println(user.greet())
            }
        }
    "#;

    let mut parser = Parser::new(source);
    if let Ok(ast) = parser.parse() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast);
        assert_snapshot!("complex_program_transpiled", rust_code);
    }
}