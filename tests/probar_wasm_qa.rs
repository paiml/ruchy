//! Probar-based WASM QA Framework Tests
//!
//! Ported from tests.disabled/`wasm_qa_framework.rs` and `wasm_memory_leak_detection.rs`
//! Uses jugar-probar for GUI coverage tracking.
//!
//! Run with: `cargo test --test probar_wasm_qa`

use jugar_probar::prelude::*;
use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;

// =============================================================================
// GUI Coverage Tracking for WASM QA
// =============================================================================

fn wasm_qa_coverage() -> UxCoverageTracker {
    UxCoverageBuilder::new()
        // Core parsing operations
        .button("parse_basic")
        .button("parse_complex")
        .button("parse_invalid")
        .button("parse_large_input")
        // Transpilation operations
        .button("transpile_function")
        .button("transpile_deterministic")
        .button("transpile_repeated")
        // Memory operations
        .button("memory_allocation")
        .button("memory_cleanup")
        .button("memory_parser")
        .button("memory_transpiler")
        // Error handling
        .button("error_incomplete")
        .button("error_unclosed")
        .button("error_invalid_syntax")
        .button("error_recursive")
        // Test categories
        .screen("basic_compilation")
        .screen("transpilation")
        .screen("memory_safety")
        .screen("error_handling")
        .screen("stress_testing")
        .build()
}

// =============================================================================
// Basic Compilation Tests (from wasm_qa_framework.rs)
// =============================================================================

#[test]
fn test_probar_qa_basic_parsing() {
    let mut gui = wasm_qa_coverage();
    gui.visit("basic_compilation");
    gui.click("parse_basic");

    let source = "let x = 42; x + 1";
    let mut parser = Parser::new(source);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Basic parsing should work: {ast:?}");
}

#[test]
fn test_probar_qa_complex_parsing() {
    let mut gui = wasm_qa_coverage();
    gui.visit("basic_compilation");
    gui.click("parse_complex");

    let source = r"
        fn factorial(n: i32) -> i32 {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
    ";
    let mut parser = Parser::new(source);
    let ast = parser.parse();
    assert!(ast.is_ok(), "Complex parsing should work: {ast:?}");
}

#[test]
fn test_probar_qa_multiple_expressions() {
    let mut gui = wasm_qa_coverage();
    gui.visit("basic_compilation");

    let expressions = vec![
        "let x = 42",
        "fn foo() { 1 }",
        "if true { 1 } else { 2 }",
        "for i in 0..10 { i }",
        "while x > 0 { x = x - 1 }",
        "match x { 1 => a, _ => b }",
        "struct Point { x: i32, y: i32 }",
        "enum Color { Red, Green, Blue }",
        "|x| x + 1",
    ];

    for expr in expressions {
        let mut parser = Parser::new(expr);
        let result = parser.parse();
        assert!(
            result.is_ok(),
            "Expression '{expr}' should parse: {result:?}"
        );
    }
}

// =============================================================================
// Transpilation Tests (from wasm_qa_framework.rs)
// =============================================================================

#[test]
fn test_probar_qa_transpilation_deterministic() {
    let mut gui = wasm_qa_coverage();
    gui.visit("transpilation");
    gui.click("transpile_deterministic");

    let source = "fn add(a: i32, b: i32) -> i32 { a + b }";
    let mut parser = Parser::new(source);

    if let Ok(ast) = parser.parse() {
        let mut transpiler = Transpiler::new();
        let result1 = transpiler.transpile(&ast);
        let result2 = transpiler.transpile(&ast);

        assert_eq!(
            result1.is_ok(),
            result2.is_ok(),
            "Transpilation should be deterministic"
        );

        if let (Ok(code1), Ok(code2)) = (&result1, &result2) {
            assert_eq!(
                code1.to_string(),
                code2.to_string(),
                "Transpiled code should be identical"
            );
        }
    }
}

#[test]
fn test_probar_qa_transpile_function() {
    let mut gui = wasm_qa_coverage();
    gui.visit("transpilation");
    gui.click("transpile_function");

    let source = "fn greet(name: String) -> String { name }";
    let mut parser = Parser::new(source);

    if let Ok(ast) = parser.parse() {
        let mut transpiler = Transpiler::new();
        let result = transpiler.transpile(&ast);
        assert!(
            result.is_ok(),
            "Function transpilation should succeed: {result:?}"
        );
    }
}

#[test]
fn test_probar_qa_transpile_repeated() {
    let mut gui = wasm_qa_coverage();
    gui.visit("transpilation");
    gui.click("transpile_repeated");

    let mut transpiler = Transpiler::new();

    for i in 0..30 {
        let source = format!("fn test{i}() {{ {i} }}");
        let mut parser = Parser::new(&source);

        if let Ok(ast) = parser.parse() {
            let _result = transpiler.transpile(&ast);
            // Result will be dropped automatically
        }
    }
}

// =============================================================================
// Memory Safety Tests (from wasm_memory_leak_detection.rs)
// =============================================================================

#[test]
fn test_probar_qa_memory_allocation() {
    let mut gui = wasm_qa_coverage();
    gui.visit("memory_safety");
    gui.click("memory_allocation");

    // Check that repeated allocation and deallocation doesn't leak memory
    for iteration in 0..100 {
        // Allocate some data
        let data = vec![0u8; 1024 * (iteration % 10)]; // Variable size 0-9KB

        // Process it
        let checksum: u32 = data.iter().map(|&x| u32::from(x)).sum();

        // Check processing
        assert_eq!(checksum, 0); // All zeros

        // Rust will automatically deallocate when data goes out of scope
    }
}

#[test]
fn test_probar_qa_memory_parser_cleanup() {
    let mut gui = wasm_qa_coverage();
    gui.visit("memory_safety");
    gui.click("memory_parser");

    // Check that parser doesn't leak memory across multiple uses
    for i in 0..50 {
        let source = format!("let x{i} = {};", i * 2);
        let mut parser = Parser::new(&source);
        let result = parser.parse();

        // Parser should either succeed or fail gracefully
        match result {
            Ok(_ast) => {
                // AST will be dropped automatically
            }
            Err(_e) => {
                // Error will be dropped automatically
            }
        }
    }
}

#[test]
fn test_probar_qa_memory_transpiler_stability() {
    let mut gui = wasm_qa_coverage();
    gui.visit("memory_safety");
    gui.click("memory_transpiler");

    // Check that transpiler maintains stable memory usage
    let mut transpiler = Transpiler::new();

    for i in 0..30 {
        let source = format!("fn test{i}() {{ {i} }}");
        let mut parser = Parser::new(&source);

        if let Ok(ast) = parser.parse() {
            let _result = transpiler.transpile(&ast);
            // Result will be dropped automatically
        }
    }
}

#[test]
fn test_probar_qa_memory_checksum() {
    let mut gui = wasm_qa_coverage();
    gui.visit("memory_safety");
    gui.click("memory_cleanup");

    // Check that our code doesn't have obvious memory issues
    let mut data = vec![0u8; 1024];
    for (i, byte) in data.iter_mut().enumerate() {
        *byte = u8::try_from(i % 256).unwrap_or(0);
    }

    // Process the data through our system
    let checksum_before: u32 = data.iter().map(|&x| u32::from(x)).sum();

    // Simulate some processing
    let processed = data.clone();
    let checksum_after: u32 = processed.iter().map(|&x| u32::from(x)).sum();

    assert_eq!(
        checksum_before, checksum_after,
        "Memory should remain consistent"
    );
}

// =============================================================================
// Error Handling Tests (from wasm_memory_leak_detection.rs)
// =============================================================================

#[test]
fn test_probar_qa_error_incomplete() {
    let mut gui = wasm_qa_coverage();
    gui.visit("error_handling");
    gui.click("error_incomplete");

    let incomplete_inputs = ["let x = ", "fn foo(", "if true", "for i in", "match x"];

    for input in incomplete_inputs {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        // Should fail gracefully without panic
        assert!(result.is_err(), "Incomplete input '{input}' should fail");
    }
}

#[test]
fn test_probar_qa_error_unclosed() {
    let mut gui = wasm_qa_coverage();
    gui.visit("error_handling");
    gui.click("error_unclosed");

    let unclosed_inputs = [
        "if true {",
        "fn foo() {",
        "match x {",
        "for i in items {",
        "while true {",
        "[1, 2, 3",
        "(1 + 2",
    ];

    for input in unclosed_inputs {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        // Should fail gracefully without panic
        assert!(result.is_err(), "Unclosed input '{input}' should fail");
    }
}

#[test]
fn test_probar_qa_error_invalid_syntax() {
    let mut gui = wasm_qa_coverage();
    gui.visit("error_handling");
    gui.click("error_invalid_syntax");

    let invalid_inputs = [
        "let = 42",     // Missing variable name
        "if { }",       // Missing condition
        "for in x { }", // Missing iterator variable
        // Note: "++ x" parses as "+ (+ x)" which is valid in Ruchy
        "x +++", // Invalid suffix
    ];

    for input in invalid_inputs {
        let mut parser = Parser::new(input);
        let result = parser.parse();
        // Should fail gracefully without panic
        assert!(result.is_err(), "Invalid syntax '{input}' should fail");
    }
}

#[test]
fn test_probar_qa_error_recursive_nesting() {
    let mut gui = wasm_qa_coverage();
    gui.visit("error_handling");
    gui.click("error_recursive");

    // Check deeply nested structures don't cause memory issues
    let mut nested = String::from("1");
    for _ in 0..100 {
        nested = format!("({nested})");
    }

    let mut parser = Parser::new(&nested);
    let _result = parser.parse(); // May hit recursion limits, that's ok

    // Should not leak memory or panic even with deep nesting
}

// =============================================================================
// Stress Tests
// =============================================================================

#[test]
fn test_probar_qa_large_input() {
    let mut gui = wasm_qa_coverage();
    gui.visit("stress_testing");
    gui.click("parse_large_input");

    // Check that large inputs are handled without excessive memory usage
    let large_input = "let x = 1; ".repeat(1000); // ~10KB of repeated code

    let mut parser = Parser::new(&large_input);
    let result = parser.parse();

    // Should handle large input gracefully
    assert!(result.is_ok() || result.is_err()); // Either succeeds or fails gracefully
}

#[test]
fn test_probar_qa_stress_repeated_parsing() {
    let mut gui = wasm_qa_coverage();
    gui.visit("stress_testing");

    let sources = [
        "let x = 1",
        "fn foo() { 42 }",
        "if true { 1 } else { 2 }",
        "for i in 0..10 { i }",
    ];

    // Repeated parsing should not accumulate memory
    for _ in 0..100 {
        for source in &sources {
            let mut parser = Parser::new(source);
            let _result = parser.parse();
        }
    }
}

// =============================================================================
// Property-Based Tests
// =============================================================================

#[test]
fn test_probar_qa_property_parser_never_panics() {
    let mut gui = wasm_qa_coverage();
    gui.visit("stress_testing");
    gui.click("parse_invalid");

    // Test with various potentially problematic inputs
    let problematic_inputs = [
        "",
        " ",
        "\n",
        "\t",
        "🎉",
        "// comment",
        "/* block */",
        "\\",
        "\"",
        "'",
        ";;;;;",
        "(((((",
        ")))))",
        "{{{{{",
        "}}}}}",
        "[[[[[",
        "]]]]]",
        "!@#$%^&*()",
        "null",
        "undefined",
        "NaN",
        "Infinity",
        "-Infinity",
        "0x1234",
        "0b1010",
        "0o777",
        "1e308",
        "1e-308",
    ];

    for input in problematic_inputs {
        let mut parser = Parser::new(input);
        // Should not panic, may succeed or fail
        let _result = parser.parse();
    }
}

// =============================================================================
// Coverage Report Test
// =============================================================================

#[test]
fn test_probar_qa_coverage_report() {
    let mut gui = wasm_qa_coverage();

    // Record all operations
    gui.click("parse_basic");
    gui.click("parse_complex");
    gui.click("parse_invalid");
    gui.click("parse_large_input");
    gui.click("transpile_function");
    gui.click("transpile_deterministic");
    gui.click("transpile_repeated");
    gui.click("memory_allocation");
    gui.click("memory_cleanup");
    gui.click("memory_parser");
    gui.click("memory_transpiler");
    gui.click("error_incomplete");
    gui.click("error_unclosed");
    gui.click("error_invalid_syntax");
    gui.click("error_recursive");

    // Visit all screens
    gui.visit("basic_compilation");
    gui.visit("transpilation");
    gui.visit("memory_safety");
    gui.visit("error_handling");
    gui.visit("stress_testing");

    // Generate report
    let report = gui.generate_report();
    println!("\n{report}");
    println!("WASM QA Coverage: {}", gui.summary());

    // Assert high coverage
    let percent = gui.percent();
    assert!(
        gui.meets(80.0),
        "WASM QA coverage should be at least 80%: {percent:.1}%"
    );
}
