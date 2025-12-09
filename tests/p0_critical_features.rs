#![allow(missing_docs)]
//! P0 CRITICAL FEATURE TESTS - MUST NEVER FAIL
//!
//! These tests represent the absolute minimum functionality that must work.
//! ANY failure here blocks commits and releases.
//!
//! Extreme TDD Principle: If it's advertised, it MUST work.

use ruchy::compile;
use std::fs;
use std::process::Command;

/// Helper to run Ruchy code and get output
fn run(code: &str) -> Result<String, String> {
    // Use unique temp file per test to avoid parallel test interference
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let tmp_file = format!("/tmp/p0_test_{}_{}.ruchy", std::process::id(), id);
    fs::write(&tmp_file, code).map_err(|e| e.to_string())?;

    // Use binary directly (built by test runner) - avoids cargo overhead
    // Falls back to cargo run if binary not found
    let binary_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::Path::to_path_buf))
        .map(|p| p.join("ruchy"))
        .filter(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from("target/debug/ruchy"));

    let output = if binary_path.exists() {
        Command::new(&binary_path)
            .args(["run", &tmp_file])
            .output()
            .map_err(|e| e.to_string())?
    } else {
        // Fallback to cargo run
        Command::new("cargo")
            .args(["run", "--bin", "ruchy", "--", "run", &tmp_file])
            .output()
            .map_err(|e| e.to_string())?
    };

    // Clean up temp file
    let _ = fs::remove_file(&tmp_file);

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// CRITICAL: Basic function compilation must work
#[test]
fn p0_basic_function_compilation() {
    let code = r"
        fn add(a: i32, b: i32) -> i32 {
            a + b
        }
        println(add(5, 3))
    ";

    let result = compile(code);
    assert!(
        result.is_ok(),
        "CRITICAL: Basic function compilation failed: {result:?}"
    );
}

/// CRITICAL: Match expressions with integer patterns
#[test]
fn p0_match_with_integers() {
    let code = r#"
        fn classify(n: i32) -> String {
            match n {
                0 => "zero"
                1 => "one"
                _ => "many"
            }
        }
        println(classify(1))
    "#;

    let result = compile(code);
    assert!(
        result.is_ok(),
        "CRITICAL: Match with integers failed: {result:?}"
    );
}

/// CRITICAL: Factorial (recursive functions)
#[test]
fn p0_recursive_factorial() {
    let code = r"
        fn factorial(n: i32) -> i32 {
            if n <= 1 { 1 } else { n * factorial(n - 1) }
        }
        println(factorial(5))
    ";

    let result = compile(code);
    assert!(
        result.is_ok(),
        "CRITICAL: Recursive factorial failed: {result:?}"
    );
}

/// CRITICAL: Fibonacci (pattern matching recursion)
#[test]
fn p0_fibonacci_pattern_match() {
    let code = r"
        fn fib(n: i32) -> i32 {
            match n {
                0 => 0
                1 => 1
                _ => fib(n - 1) + fib(n - 2)
            }
        }
        println(fib(10))
    ";

    let result = compile(code);
    assert!(
        result.is_ok(),
        "CRITICAL: Fibonacci pattern match failed: {result:?}"
    );
}

/// CRITICAL: String operations
#[test]
fn p0_string_concatenation() {
    let code = r#"
        let name = "World"
        let greeting = "Hello, " + name
        println(greeting)
    "#;

    let result = run(code);
    assert!(
        result.is_ok(),
        "CRITICAL: String concatenation failed: {result:?}"
    );
    assert!(
        result.unwrap().contains("Hello, World"),
        "String output incorrect"
    );
}

/// CRITICAL: For loops
#[test]
fn p0_for_loop() {
    let code = r"
        for i in 1..=3 {
            println(i)
        }
    ";

    let result = run(code);
    assert!(result.is_ok(), "CRITICAL: For loop failed: {result:?}");
    let output = result.unwrap();
    assert!(output.contains('1') && output.contains('2') && output.contains('3'));
}

/// CRITICAL: Arrays
#[test]
fn p0_array_operations() {
    let code = r"
        let arr = [1, 2, 3, 4, 5]
        println(arr[2])
    ";

    let result = run(code);
    assert!(
        result.is_ok(),
        "CRITICAL: Array operations failed: {result:?}"
    );
    assert!(result.unwrap().contains('3'));
}

/// CRITICAL: While loops
#[test]
fn p0_while_loop() {
    let code = r"
        let mut count = 0
        while count < 3 {
            count = count + 1
        }
        println(count)
    ";

    let result = run(code);
    assert!(result.is_ok(), "CRITICAL: While loop failed: {result:?}");
    assert!(result.unwrap().contains('3'));
}

/// CRITICAL: If-else statements
#[test]
fn p0_if_else() {
    let code = r#"
        let x = 10
        if x > 5 {
            println("big")
        } else {
            println("small")
        }
    "#;

    let result = run(code);
    assert!(result.is_ok(), "CRITICAL: If-else failed: {result:?}");
    assert!(result.unwrap().contains("big"));
}

/// CRITICAL: Function with no return value should not generate `HashSet`
#[test]
fn p0_no_hashset_in_functions() {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;

    let code = "fn compute(a: i32, b: i32) -> i32 { a + b }";
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Parse failed");

    let mut transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Transpile failed");
    let rust_str = rust_code.to_string();

    assert!(
        !rust_str.contains("HashSet"),
        "CRITICAL: Functions generating HashSet code! Generated: {rust_str}"
    );
}

// ===== ACTOR TESTS (Currently Expected to Fail) =====

#[test]
#[ignore = "Actor runtime not implemented - tracking issue"]
fn p0_actor_definition() {
    let code = r"
        actor Counter {
            count: i32
        }

        let c = Counter { count: 0 }
        println(c.count)
    ";

    let result = run(code);
    assert!(
        result.is_ok(),
        "CRITICAL: Actor definition failed: {result:?}"
    );
}

#[test]
#[ignore = "Actor message passing not implemented"]
fn p0_actor_messages() {
    let code = r#"
        actor Echo {
            receive {
                msg: String => println("Echo: " + msg)
            }
        }

        let echo = spawn Echo {}
        echo.send("Hello")
    "#;

    let result = run(code);
    assert!(
        result.is_ok(),
        "CRITICAL: Actor messaging failed: {result:?}"
    );
}

// ===== STRUCT/CLASS TESTS (Currently Expected to Fail) =====

#[test]
#[ignore = "Struct runtime not implemented - tracking issue"]
fn p0_struct_definition() {
    let code = r"
        struct Point {
            x: i32,
            y: i32
        }

        let p = Point { x: 10, y: 20 }
        println(p.x)
    ";

    let result = run(code);
    assert!(
        result.is_ok(),
        "CRITICAL: Struct definition failed: {result:?}"
    );
    assert!(result.unwrap().contains("10"));
}

#[test]
#[ignore = "Class runtime not implemented - tracking issue"]
fn p0_class_definition() {
    let code = r"
        class Rectangle {
            width: i32,
            height: i32

            fn area(self) -> i32 {
                self.width * self.height
            }
        }

        let r = Rectangle { width: 5, height: 10 }
        println(r.area())
    ";

    let result = run(code);
    assert!(
        result.is_ok(),
        "CRITICAL: Class definition failed: {result:?}"
    );
    assert!(result.unwrap().contains("50"));
}

// ===== TRANSPILER CONSISTENCY TESTS =====

#[test]
fn p0_transpiler_deterministic() {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;

    let code = "fn add(a: i32, b: i32) -> i32 { a + b }";

    // Transpile twice
    let mut parser1 = Parser::new(code);
    let ast1 = parser1.parse().expect("Parse failed");
    let mut transpiler1 = Transpiler::new();
    let result1 = transpiler1
        .transpile(&ast1)
        .expect("Transpile failed")
        .to_string();

    let mut parser2 = Parser::new(code);
    let ast2 = parser2.parse().expect("Parse failed");
    let mut transpiler2 = Transpiler::new();
    let result2 = transpiler2
        .transpile(&ast2)
        .expect("Transpile failed")
        .to_string();

    assert_eq!(result1, result2, "CRITICAL: Transpiler not deterministic!");
}

/// Property test: ALL arithmetic operators must compile
#[test]
fn p0_all_arithmetic_operators() {
    let operators = vec![
        ("+", "addition"),
        ("-", "subtraction"),
        ("*", "multiplication"),
        ("/", "division"),
        ("%", "modulo"),
    ];

    for (op, name) in operators {
        let code =
            format!("fn compute(a: i32, b: i32) -> i32 {{ a {op} b }}\nprintln(compute(10, 3))");

        let result = compile(&code);
        assert!(
            result.is_ok(),
            "CRITICAL: {name} operator ({op}) failed: {result:?}"
        );
    }
}

/// Property test: ALL comparison operators must work
#[test]
fn p0_all_comparison_operators() {
    let operators = vec![
        ("==", "equality"),
        ("!=", "inequality"),
        ("<", "less than"),
        (">", "greater than"),
        ("<=", "less or equal"),
        (">=", "greater or equal"),
    ];

    for (op, name) in operators {
        let code = format!("let result = 5 {op} 3\nprintln(result)");

        let result = run(&code);
        assert!(
            result.is_ok(),
            "CRITICAL: {name} operator ({op}) failed: {result:?}"
        );
    }
}

/// CRITICAL: Book example compilation
#[test]
fn p0_book_examples_compile() {
    // Examples that MUST work from the book
    let examples = [
        // Chapter 5: Functions
        r#"
        fun greet(name: String) -> String {
            "Hello, " + name + "!"
        }
        println(greet("World"))
        "#,
        // Chapter 6: Control Flow
        r#"
        let score = 85
        let grade = if score >= 90 {
            "A"
        } else if score >= 80 {
            "B"
        } else {
            "C"
        }
        println(grade)
        "#,
        // Chapter 7: Pattern Matching
        r#"
        let day = 3
        let day_name = match day {
            1 => "Monday"
            2 => "Tuesday"
            3 => "Wednesday"
            _ => "Other"
        }
        println(day_name)
        "#,
    ];

    for (i, example) in examples.iter().enumerate() {
        let result = compile(example);
        assert!(
            result.is_ok(),
            "CRITICAL: Book example {} failed to compile: {:?}",
            i + 1,
            result
        );
    }
}

/// Test to ensure we catch `HashSet` generation early
#[test]
fn p0_detect_hashset_regression() {
    use ruchy::backend::transpiler::Transpiler;
    use ruchy::frontend::parser::Parser;

    // Various function patterns that should NEVER generate HashSet
    let test_cases = vec![
        ("fn id(x: i32) -> i32 { x }", "identity"),
        ("fn add(a: i32, b: i32) -> i32 { a + b }", "addition"),
        ("fn neg(x: i32) -> i32 { -x }", "negation"),
        ("fn mul(x: i32, y: i32) -> i32 { x * y }", "multiplication"),
        (
            "fn max(a: i32, b: i32) -> i32 { if a > b { a } else { b } }",
            "conditional",
        ),
    ];

    for (code, name) in test_cases {
        let mut parser = Parser::new(code);
        let ast = parser
            .parse()
            .unwrap_or_else(|_| panic!("Parse failed for {name}"));

        let mut transpiler = Transpiler::new();
        let rust_code = transpiler
            .transpile(&ast)
            .unwrap_or_else(|_| panic!("Transpile failed for {name}"));
        let rust_str = rust_code.to_string();

        assert!(
            !rust_str.contains("HashSet"),
            "CRITICAL: {name} function generated HashSet! Code: {rust_str}"
        );
        assert!(
            !rust_str.contains("__temp_set"),
            "CRITICAL: {name} function has temp set! Code: {rust_str}"
        );
    }
}
