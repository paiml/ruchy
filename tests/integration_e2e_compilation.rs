//! End-to-End Compilation Integration Tests (QUALITY-009 Phase 1)
//! 
//! Tests complete compilation workflows from source to execution,
//! validating cross-module functionality that unit tests cannot cover.

#![allow(warnings)] // Test file - contains Ruchy code in strings

use ruchy::{Parser, Transpiler};

/// Integration test harness for end-to-end compilation testing
struct E2ETestHarness {
    transpiler: Transpiler,
}

impl E2ETestHarness {
    fn new() -> Self {
        Self {
            transpiler: Transpiler::new(),
        }
    }

    /// Compile a single Ruchy program and verify it transpiles successfully
    fn compile_program(&self, source: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(source);
        let ast = parser.parse()?;
        let result = self.transpiler.transpile(&ast)?;
        Ok(result.to_string())
    }


    /// Validate that transpiled code contains expected patterns
    fn validate_output(&self, transpiled: &str, expected_patterns: &[&str]) -> bool {
        expected_patterns.iter().all(|pattern| {
            transpiled.contains(pattern)
        })
    }
}

#[test]
fn test_single_file_hello_world() {
    let harness = E2ETestHarness::new();
    
    let source = r"
        fun main() {
            println("Hello, World!")
        }
    ";

    let result = harness.compile_program(source)
        .expect("Should compile simple hello world program");
    
    // Validate essential patterns are present in transpiled code
    let expected_patterns = ["fn main", "println", "Hello, World!"];
    assert!(
        harness.validate_output(&result, &expected_patterns),
        "Transpiled code missing expected patterns. Got: {}", result
    );
}

#[test] 
fn test_single_file_with_functions() {
    let harness = E2ETestHarness::new();
    
    let source = r"
        fun add(a: i32, b: i32) -> i32 {
            a + b
        }
        
        fun main() {
            let result = add(2, 3)
            println(result)
        }
    ";

    let result = harness.compile_program(source)
        .expect("Should compile program with functions");
    
    let expected_patterns = ["fn add", "a : i32", "b : i32", "-> i32", "fn main", "let result"];
    assert!(
        harness.validate_output(&result, &expected_patterns),
        "Transpiled code missing function patterns. Got: {}", result
    );
}

#[test]
fn test_single_file_with_match_expressions() {
    let harness = E2ETestHarness::new();
    
    let source = r"
        fun classify(x: i32) -> str {
            match x {
                0 => "zero",
                1 => "one", 
                _ => "other"
            }
        }
        
        fun main() {
            println(classify(1))
        }
    ";

    let result = harness.compile_program(source)
        .expect("Should compile program with match expressions");
    
    let expected_patterns = ["match", "0i32 =>", "1i32 =>", "_ =>", "zero", "one", "other"];
    assert!(
        harness.validate_output(&result, &expected_patterns),
        "Transpiled code missing match patterns. Got: {}", result
    );
}

#[test]
fn test_single_file_with_loops() {
    let harness = E2ETestHarness::new();
    
    let source = r"
        fun main() {
            let mut i = 0
            while i < 5 {
                println(i)
                i = i + 1
            }
            
            for x in [1, 2, 3] {
                println(x)
            }
        }
    ";

    let result = harness.compile_program(source)
        .expect("Should compile program with loops");
    
    let expected_patterns = ["while", "for", "in", "vec !"];
    assert!(
        harness.validate_output(&result, &expected_patterns),
        "Transpiled code missing loop patterns. Got: {}", result
    );
}

#[test]
fn test_single_file_with_data_structures() {
    let harness = E2ETestHarness::new();
    
    let source = r"
        fun main() {
            let tuple = (1, "hello", true)
            let array = [1, 2, 3, 4, 5]
            let obj = { name: "test", value: 42 }
            
            println(tuple)
            println(array)
            println(obj)
        }
    ";

    let result = harness.compile_program(source)
        .expect("Should compile program with data structures");
    
    let expected_patterns = ["tuple", "array", "obj", "HashMap"];
    assert!(
        harness.validate_output(&result, &expected_patterns),
        "Transpiled code missing data structure patterns. Got: {}", result
    );
}

#[test]
fn test_pattern_destructuring_integration() {
    let harness = E2ETestHarness::new();
    
    // Test the newly implemented pattern destructuring from QUALITY-007
    let source = r"
        fun main() {
            let (a, b) = (1, 2)
            let [first, ..rest] = [1, 2, 3, 4, 5]
            
            match (a, b) {
                (1, 2) => println("matched tuple"),
                _ => println("no match")
            }
        }
    ";

    let result = harness.compile_program(source)
        .expect("Should compile program with pattern destructuring");
    
    let expected_patterns = ["let (a , b)", "let [first", "rest]", "match", "(1i32 , 2i32)"];
    assert!(
        harness.validate_output(&result, &expected_patterns),
        "Transpiled code missing pattern destructuring. Got: {}", result
    );
}

#[test] 
fn test_error_handling_compilation() {
    let harness = E2ETestHarness::new();
    
    let source = r"
        fun divide(a: i32, b: i32) -> i32 {
            if b == 0 {
                0
            } else {
                a / b
            }
        }
        
        fun main() {
            let result = divide(10, 2)
            println(result)
        }
    ";

    let result = harness.compile_program(source)
        .expect("Should compile program with error handling");
    
    let expected_patterns = ["fn divide", "if", "else", "fn main"];
    assert!(
        harness.validate_output(&result, &expected_patterns),
        "Transpiled code missing error handling patterns. Got: {}", result
    );
}

#[test]
fn test_string_interpolation_compilation() {
    let harness = E2ETestHarness::new();
    
    let source = r"
        fun main() {
            let name = "World"
            let count = 42
            println(f"Hello, {name}! Count: {count}")
        }
    ";

    let result = harness.compile_program(source)
        .expect("Should compile program with string interpolation");
    
    let expected_patterns = ["Hello", "Count:", "name", "count"];
    assert!(
        harness.validate_output(&result, &expected_patterns),
        "Transpiled code missing string interpolation patterns. Got: {}", result
    );
}

#[test]
fn test_comprehensive_language_features() {
    let harness = E2ETestHarness::new();
    
    // Comprehensive test combining multiple language features
    let source = r"
        fun fibonacci(n: i32) -> i32 {
            match n {
                0 => 0,
                1 => 1,
                _ => fibonacci(n - 1) + fibonacci(n - 2)
            }
        }
        
        fun main() {
            let numbers = [1, 2, 3, 4, 5]
            let mut results = []
            
            for num in numbers {
                let fib = fibonacci(num)
                results = results + [fib]
                println(f"fib({num}) = {fib}")
            }
            
            let first = results[0]
            println(f"First: {first}")
        }
    ";

    let result = harness.compile_program(source)
        .expect("Should compile comprehensive program");
    
    let expected_patterns = [
        "fn fibonacci", "match", "fn main", "for", "in", 
        "first", "fib(", "First:"
    ];
    assert!(
        harness.validate_output(&result, &expected_patterns),
        "Transpiled code missing comprehensive patterns. Got: {}", result
    );
}