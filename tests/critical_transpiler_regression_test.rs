//! CRITICAL: Extreme TDD Test for v3.51.0 Transpiler Regression
//!
//! This test ensures the transpiler correctly handles function bodies
//! and does NOT generate HashSet code for simple return expressions.

use ruchy::compile;

#[test]
fn test_simple_function_add_transpilation() {
    // This is the exact failing example from the book
    let code = r#"
        fun add(a: i32, b: i32) -> i32 {
            a + b
        }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile simple add function: {:?}",
        result
    );
}

#[test]
fn test_function_with_arithmetic_expression() {
    let code = r#"
        fun calculate(x: i32, y: i32) -> i32 {
            x * 2 + y
        }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile arithmetic function: {:?}",
        result
    );
}

#[test]
fn test_factorial_function() {
    let code = r#"
        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile factorial function: {:?}",
        result
    );
}

#[test]
fn test_function_with_block_return() {
    let code = r#"
        fn double(x: i32) -> i32 {
            let result = x * 2
            result
        }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile function with block: {:?}",
        result
    );
}

#[test]
fn test_function_with_single_expression() {
    let code = r#"
        fn square(x: i32) -> i32 { x * x }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile single-expression function: {:?}",
        result
    );
}

#[test]
fn test_function_with_string_return() {
    let code = r#"
        fn greet(name: &str) -> String {
            "Hello " + name
        }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile string function: {:?}",
        result
    );
}

#[test]
fn test_fibonacci_function() {
    let code = r#"
        fn fibonacci(n: i32) -> i32 {
            match n {
                0 => 0
                1 => 1
                _ => fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile fibonacci function: {:?}",
        result
    );
}

#[test]
fn test_function_with_conditionals() {
    let code = r#"
        fn max(a: i32, b: i32) -> i32 {
            if a > b { a } else { b }
        }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile max function: {:?}",
        result
    );
}

#[test]
fn test_real_set_literal_still_works() {
    // Make sure we didn't break actual set literals
    let code = r#"
        let my_set = {1, 2, 3}
        println(my_set)
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile set literal: {:?}",
        result
    );
}

#[test]
fn test_empty_function_body() {
    let code = r#"
        fn noop() { }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile empty function: {:?}",
        result
    );
}

/// Property test: ANY function with a return type and simple expression should compile
#[test]
fn test_property_all_arithmetic_functions_compile() {
    let operators = vec!["+", "-", "*", "/", "%"];
    for op in operators {
        let code = format!(
            r#"
            fn compute(a: i32, b: i32) -> i32 {{
                a {} b
            }}
            "#,
            op
        );

        let result = compile(&code);
        assert!(
            result.is_ok(),
            "Failed to compile function with operator {}: {:?}",
            op,
            result
        );
    }
}

/// Extreme edge case: deeply nested expressions should still work
#[test]
fn test_deeply_nested_expression() {
    let code = r#"
        fn nested(x: i32) -> i32 {
            ((((x + 1) * 2) - 3) / 4) % 5
        }
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Failed to compile nested expression: {:?}",
        result
    );
}

/// Test the exact example from the book that was failing
#[test]
fn test_book_example_chapter_5_2() {
    let code = r#"
        fun add(a: i32, b: i32) -> i32 {
            a + b
        }

        println(add(5, 3))
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "Book example 5.2 failed to compile: {:?}",
        result
    );
}

/// Test that the transpiler doesn't generate HashSet code
#[test]
fn test_transpiler_output_no_hashset() {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;

    let code = "fun add(a: i32, b: i32) -> i32 { a + b }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Failed to parse");

    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Failed to transpile");
    let rust_str = rust_code.to_string();

    // The generated code should NOT contain HashSet
    assert!(
        !rust_str.contains("HashSet"),
        "Transpiler incorrectly generated HashSet code: {}",
        rust_str
    );

    // The generated code SHOULD contain simple return
    assert!(
        rust_str.contains("a + b"),
        "Transpiler didn't generate simple arithmetic: {}",
        rust_str
    );
}
