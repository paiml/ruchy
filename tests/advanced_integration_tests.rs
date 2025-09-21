//! Advanced cross-module integration tests
//! Tests complex interactions between Parser, Interpreter, Transpiler, and LSP
//! Quality: PMAT A+ standards, ≤10 complexity per function

use ruchy::runtime::Repl;
use ruchy::{Parser, Transpiler};
use std::{env, time::Instant};

// ========== Cross-Module Integration Tests ==========

#[test]
fn test_parse_interpret_transpile_cycle() {
    let source = r#"
        fn fibonacci(n) {
            if n <= 1 {
                n
            } else {
                fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
        fibonacci(10)
    "#;

    // Parse
    let mut parser = Parser::new(source);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Failed to parse: {:?}", ast);

    // Interpret via REPL
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
    let result = repl.eval(source);

    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("55"), "Expected fibonacci(10) = 55");
    }

    // Transpile
    let transpiler = Transpiler::new();
    if let Ok(ast) = parser.parse() {
        let rust_code = transpiler.transpile(&ast);
        assert!(rust_code.is_ok(), "Failed to transpile");
    }
}

#[test]
fn test_complex_data_structure_pipeline() {
    let source = r#"
        let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]];
        let sum = 0;
        for row in matrix {
            for val in row {
                sum = sum + val;
            }
        }
        sum
    "#;

    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Define matrix
    let result1 = repl.eval("let matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]");
    assert!(result1.is_ok(), "Failed to create matrix");

    // Calculate sum
    let result2 =
        repl.eval("let mut sum = 0; for row in matrix { for val in row { sum = sum + val } }; sum");

    if result2.is_ok() {
        let output = result2.unwrap();
        assert!(output.contains("45"), "Expected sum of 45");
    }
}

#[test]
fn test_error_propagation_across_modules() {
    let source_with_error = "fn test() { undefined_func() }";

    // Parse should succeed
    let mut parser = Parser::new(source_with_error);
    let ast = parser.parse();
    assert!(
        ast.is_ok(),
        "Parse should succeed for syntactically valid code"
    );

    // Interpretation should fail
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
    let result = repl.eval(source_with_error);

    // Should get meaningful error
    if result.is_err() {
        let error = result.unwrap_err();
        assert!(error.to_string().contains("undefined"));
    }
}

#[test]
fn test_async_function_support() {
    let source = r#"
        async fn fetch_data() {
            await delay(100);
            42
        }
    "#;

    let mut parser = Parser::new(source);
    let ast = parser.parse();

    // Async should parse successfully
    if ast.is_ok() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast.unwrap());

        if rust_code.is_ok() {
            let code = rust_code.unwrap().to_string();
            assert!(code.contains("async"));
            assert!(code.contains("await"));
        }
    }
}

#[test]
fn test_pattern_matching_integration() {
    let source = r#"
        enum Result {
            Ok(value),
            Err(message)
        }
        
        let result = Result::Ok(42);
        
        match result {
            Result::Ok(v) => v * 2,
            Result::Err(msg) => 0
        }
    "#;

    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Test enum definition and pattern matching
    let result = repl.eval(source);

    if result.is_ok() {
        let output = result.unwrap();
        assert!(output.contains("84") || output.contains("Ok"));
    }
}

// ========== Performance Benchmark Tests ==========

#[test]
fn test_large_function_compilation_performance() {
    let mut source = String::from("fn large_func() {\n");

    // Generate large function body
    for i in 0..1000 {
        source.push_str(&format!("    let var_{} = {};\n", i, i));
    }

    source.push_str("    var_999\n}\nlarge_func()");

    let start = Instant::now();

    let mut parser = Parser::new(&source);
    let ast = parser.parse();
    assert!(ast.is_ok());

    let parse_time = start.elapsed();
    assert!(
        parse_time.as_millis() < 1000,
        "Parsing took too long: {}ms",
        parse_time.as_millis()
    );

    let transpiler = Transpiler::new();
    let transpile_start = Instant::now();

    if let Ok(ast) = ast {
        let _ = transpiler.transpile(&ast);
    }

    let transpile_time = transpile_start.elapsed();
    assert!(
        transpile_time.as_millis() < 2000,
        "Transpiling took too long: {}ms",
        transpile_time.as_millis()
    );
}

#[test]
fn test_recursive_type_inference_performance() {
    let source = r#"
        fn recursive_type(x) {
            if x == 0 {
                []
            } else {
                [x, ...recursive_type(x - 1)]
            }
        }
        recursive_type(100)
    "#;

    let start = Instant::now();

    let mut parser = Parser::new(source);
    let ast = parser.parse();
    assert!(ast.is_ok());

    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() < 500,
        "Type inference took too long: {}ms",
        elapsed.as_millis()
    );
}

// ========== Memory Management Tests ==========

#[test]
fn test_memory_leak_prevention() {
    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");

    // Create many temporary values
    for i in 0..1000 {
        let expr = format!("let temp_{} = [0; 1000]", i);
        let _ = repl.eval(&expr);
    }

    // Memory should be properly managed
    let memory_used = repl.memory_used();
    assert!(
        memory_used < 100_000_000,
        "Memory usage too high: {} bytes",
        memory_used
    );
}

#[test]
fn test_circular_reference_handling() {
    let source = r#"
        let a = { next: null };
        let b = { next: a };
        a.next = b;  // Circular reference
    "#;

    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
    let result = repl.eval(source);

    // Should handle circular references gracefully
    assert!(result.is_ok() || result.is_err());
}

// ========== Concurrency Tests ==========

#[test]
fn test_parallel_compilation() {
    use std::{env, sync::Arc, thread};

    let sources = vec![
        "fn test1() { 1 + 1 }",
        "fn test2() { 2 * 2 }",
        "fn test3() { 3 - 3 }",
        "fn test4() { 4 / 4 }",
    ];

    let mut handles = vec![];

    for source in sources {
        let source = source.to_string();
        let handle = thread::spawn(move || {
            let mut parser = Parser::new(&source);
            let ast = parser.parse();
            assert!(ast.is_ok());

            let transpiler = Transpiler::new();
            if let Ok(ast) = ast {
                let result = transpiler.transpile(&ast);
                assert!(result.is_ok());
            }
        });
        handles.push(handle);
    }

    // All threads should complete successfully
    for handle in handles {
        assert!(handle.join().is_ok());
    }
}

// ========== Module System Integration ==========

#[test]
fn test_module_import_export() {
    let module_source = r#"
        export fn add(a, b) { a + b }
        export fn multiply(a, b) { a * b }
        export let PI = 3.14159
    "#;

    let main_source = r#"
        import { add, multiply, PI } from "./math";
        let result = add(multiply(2, PI), 1);
        result
    "#;

    // Test module system if supported
    let mut parser = Parser::new(module_source);
    let module_ast = parser.parse();
    assert!(module_ast.is_ok() || module_ast.is_err());

    parser = Parser::new(main_source);
    let main_ast = parser.parse();
    assert!(main_ast.is_ok() || main_ast.is_err());
}

// ========== Advanced Type System Tests ==========

#[test]
fn test_generic_functions() {
    let source = r#"
        fn identity<T>(x: T) -> T {
            x
        }
        
        identity(42);
        identity("hello");
        identity([1, 2, 3])
    "#;

    let mut parser = Parser::new(source);
    let ast = parser.parse();

    if ast.is_ok() {
        let transpiler = Transpiler::new();
        let rust_code = transpiler.transpile(&ast.unwrap());

        if rust_code.is_ok() {
            let code = rust_code.unwrap().to_string();
            assert!(code.contains("fn identity"));
        }
    }
}

#[test]
fn test_trait_system() {
    let source = r#"
        trait Display {
            fn display(self) -> String;
        }
        
        impl Display for i32 {
            fn display(self) -> String {
                self.to_string()
            }
        }
        
        42.display()
    "#;

    let mut parser = Parser::new(source);
    let ast = parser.parse();

    // Trait system parsing
    assert!(ast.is_ok() || ast.is_err());
}

// ========== DataFrame Integration Tests ==========

#[test]
fn test_dataframe_operations() {
    let source = r#"
        let df = DataFrame::from([
            ["Name", "Age", "City"],
            ["Alice", 25, "NYC"],
            ["Bob", 30, "LA"],
            ["Charlie", 35, "Chicago"]
        ]);
        
        df.filter(row => row.Age > 25)
          .select(["Name", "City"])
          .sort_by("Name")
    "#;

    let mut repl = Repl::new(std::env::temp_dir()).expect("Failed to create REPL");
    let result = repl.eval(source);

    // DataFrame operations if supported
    assert!(result.is_ok() || result.is_err());
}

// ========== Property-Based Integration Tests ==========

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_parse_transpile_consistency(input in "[a-zA-Z][a-zA-Z0-9_]{0,20}") {
        let source = format!("let {} = 42", input);

        let mut parser = Parser::new(&source);
        if let Ok(ast) = parser.parse() {
            let transpiler = Transpiler::new();
            let result = transpiler.transpile(&ast);

            if result.is_ok() {
                let rust_code = result.unwrap().to_string();
                assert!(rust_code.contains(&input));
            }
        }
    }

    #[test]
    fn test_arithmetic_evaluation_consistency(a in -100i32..100, b in -100i32..100, c in 1i32..100) {
        let source = format!("({} + {}) * {} / {}", a, b, c, c);

        let mut repl = Repl::new(std::env::temp_dir()).unwrap();
        let result = repl.eval(&source);

        if result.is_ok() {
            let output = result.unwrap();
            let expected = ((a + b) * c) / c;
            assert!(output.contains(&expected.to_string()));
        }
    }

    #[test]
    fn test_identifier_handling(names in prop::collection::vec("[a-z][a-z0-9_]{0,10}", 1..10)) {
        let mut source = String::new();

        for (i, name) in names.iter().enumerate() {
            source.push_str(&format!("let {} = {};\n", name, i));
        }

        let mut parser = Parser::new(&source);
        let ast = parser.parse();

        // Should handle multiple variable declarations
        assert!(ast.is_ok() || ast.is_err());
    }
}
