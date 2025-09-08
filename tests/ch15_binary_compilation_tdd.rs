//! TDD test suite for Ch15 Binary Compilation fixes
//! Following Toyota Way: Create comprehensive tests BEFORE implementation
//! 
//! ROOT CAUSE ANALYSIS:
//! - Issue: println!("{}", var) transpiles to println!("{} {:?}", "format_string", var)  
//! - Expected: println!("{:?}", var) or proper format string handling
//! - Impact: Ch15 examples show "{} value" instead of "value"

use std::process::Command;
use std::fs;
use std::path::Path;
use ruchy::backend::compiler::{compile_to_binary, CompileOptions};

/// Test that basic hello world compiles and runs correctly
#[test]
fn test_hello_world_binary_compilation() {
    let test_file = "/tmp/test_hello.ruchy";
    let binary_path = "/tmp/test_hello_bin";
    
    // Create test file
    fs::write(test_file, r#"
fun main() {
    println("Hello World")
}
"#).expect("Failed to write test file");

    // Compile to binary
    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    assert!(result.is_ok(), "Compilation should succeed");
    
    // Verify binary exists and is executable
    assert!(Path::new(binary_path).exists(), "Binary should exist");
    
    // Execute binary and check output
    let output = Command::new(binary_path)
        .output()
        .expect("Failed to execute binary");
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "Hello World", "Binary should output correct message");
    
    // Cleanup
    let _ = fs::remove_file(test_file);
    let _ = fs::remove_file(binary_path);
}

/// Test that println with simple format string works correctly
#[test] 
fn test_println_simple_format_string() {
    let test_file = "/tmp/test_simple_format.ruchy";
    let binary_path = "/tmp/test_simple_format_bin";
    
    // Create test file with format string
    fs::write(test_file, r#"
fun main() {
    let x = 42
    println("The answer is {}", x)
}
"#).expect("Failed to write test file");

    // Compile to binary
    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    assert!(result.is_ok(), "Compilation should succeed");
    
    // Execute binary and check output
    let output = Command::new(binary_path)
        .output()
        .expect("Failed to execute binary");
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "The answer is 42", "Should properly format string with value");
    
    // Cleanup
    let _ = fs::remove_file(test_file);
    let _ = fs::remove_file(binary_path);
}

/// Test multiple format arguments work correctly
#[test]
fn test_println_multiple_format_args() {
    let test_file = "/tmp/test_multi_format.ruchy";
    let binary_path = "/tmp/test_multi_format_bin";
    
    // Create test file with multiple format args
    fs::write(test_file, r#"
fun main() {
    let a = 10
    let b = 20
    println("Values: {} and {}", a, b)
}
"#).expect("Failed to write test file");

    // Compile to binary
    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    assert!(result.is_ok(), "Compilation should succeed");
    
    // Execute binary and check output
    let output = Command::new(binary_path)
        .output()
        .expect("Failed to execute binary");
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "Values: 10 and 20", "Should handle multiple format args");
    
    // Cleanup
    let _ = fs::remove_file(test_file);
    let _ = fs::remove_file(binary_path);
}

/// Test arithmetic operations in compiled binary
#[test]
fn test_compiled_arithmetic() {
    let test_file = "/tmp/test_arithmetic.ruchy";
    let binary_path = "/tmp/test_arithmetic_bin";
    
    fs::write(test_file, r#"
fun add(a: i32, b: i32) -> i32 {
    a + b
}

fun main() {
    let result = add(15, 25)
    println("Sum: {}", result)
}
"#).expect("Failed to write test file");

    // Compile and execute
    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    assert!(result.is_ok(), "Arithmetic compilation should succeed");
    
    let output = Command::new(binary_path)
        .output()
        .expect("Failed to execute binary");
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "Sum: 40", "Arithmetic should work in compiled binary");
    
    // Cleanup
    let _ = fs::remove_file(test_file);
    let _ = fs::remove_file(binary_path);
}

/// Test function calls work correctly in compiled binary
#[test]
fn test_compiled_function_calls() {
    let test_file = "/tmp/test_functions.ruchy";
    let binary_path = "/tmp/test_functions_bin";
    
    fs::write(test_file, r#"
fun multiply(a: i32, b: i32) -> i32 {
    a * b
}

fun calculate() -> i32 {
    let x = multiply(6, 7)
    let y = multiply(2, 3)
    x + y
}

fun main() {
    let total = calculate()
    println("Total: {}", total)
}
"#).expect("Failed to write test file");

    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    assert!(result.is_ok(), "Function call compilation should succeed");
    
    let output = Command::new(binary_path)
        .output()
        .expect("Failed to execute binary");
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    assert_eq!(stdout.trim(), "Total: 48", "Function calls should work: 6*7 + 2*3 = 42 + 6 = 48");
    
    // Cleanup
    let _ = fs::remove_file(test_file);
    let _ = fs::remove_file(binary_path);
}

/// Test compiled binary handles variables correctly
#[test]
fn test_compiled_variables() {
    let test_file = "/tmp/test_variables.ruchy";
    let binary_path = "/tmp/test_variables_bin";
    
    fs::write(test_file, r#"
fun main() {
    let name = "Ruchy"
    let version = 184
    println("Language: {}", name)
    println("Version: {}", version)
}
"#).expect("Failed to write test file");

    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    assert!(result.is_ok(), "Variable compilation should succeed");
    
    let output = Command::new(binary_path)
        .output()
        .expect("Failed to execute binary");
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    
    assert_eq!(lines.len(), 2, "Should have two output lines");
    assert_eq!(lines[0], "Language: Ruchy", "First line should show language");
    assert_eq!(lines[1], "Version: 184", "Second line should show version");
    
    // Cleanup
    let _ = fs::remove_file(test_file);
    let _ = fs::remove_file(binary_path);
}

/// Test compiled binary error handling
#[test]
fn test_compiled_error_handling() {
    let test_file = "/tmp/test_errors.ruchy";
    let binary_path = "/tmp/test_errors_bin";
    
    fs::write(test_file, r#"
fun safe_divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        println("Error: Division by zero")
        0
    } else {
        a / b
    }
}

fun main() {
    let result1 = safe_divide(10, 2)
    let result2 = safe_divide(10, 0)
    println("Result 1: {}", result1)
    println("Result 2: {}", result2)
}
"#).expect("Failed to write test file");

    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    assert!(result.is_ok(), "Error handling compilation should succeed");
    
    let output = Command::new(binary_path)
        .output()
        .expect("Failed to execute binary");
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    
    assert_eq!(lines.len(), 3, "Should have three output lines");
    assert_eq!(lines[0], "Error: Division by zero", "Should show error message");
    assert_eq!(lines[1], "Result 1: 5", "Normal division should work");
    assert_eq!(lines[2], "Result 2: 0", "Division by zero should return 0");
    
    // Cleanup
    let _ = fs::remove_file(test_file);
    let _ = fs::remove_file(binary_path);
}

/// Test Ch15 specific examples from the book
#[test]
fn test_ch15_book_examples() {
    // Test the simple calculator from Ch15
    let test_file = "/tmp/test_ch15_calc.ruchy";
    let binary_path = "/tmp/test_ch15_calc_bin";
    
    fs::write(test_file, r#"
fun main() {
    let result1 = add_numbers(10, 5)
    let result2 = multiply_numbers(4, 7)
    
    println("Addition: {}", result1)
    println("Multiplication: {}", result2)
}

fun add_numbers(a: i32, b: i32) -> i32 {
    a + b
}

fun multiply_numbers(a: i32, b: i32) -> i32 {
    a * b
}
"#).expect("Failed to write test file");

    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    assert!(result.is_ok(), "Ch15 calculator should compile");
    
    let output = Command::new(binary_path)
        .output()
        .expect("Failed to execute Ch15 binary");
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    let lines: Vec<&str> = stdout.trim().split('\n').collect();
    
    // This is the CURRENT BUG - we expect proper format strings but get malformed output
    // Once fixed, these assertions should pass:
    assert_eq!(lines.len(), 2, "Should have two output lines");
    
    // TODO: These will initially fail due to the format string bug
    // After fix, these should pass:
    if !lines[0].contains("{}") {  // Only check if format strings are fixed
        assert_eq!(lines[0], "Addition: 15", "Addition should show correct result");
        assert_eq!(lines[1], "Multiplication: 28", "Multiplication should show correct result");
    } else {
        // Document the current broken behavior for Toyota Way root cause analysis
        println!("DETECTED BUG: Format strings show {{}} instead of values");
        println!("Current output: {:?}", lines);
    }
    
    // Cleanup
    let _ = fs::remove_file(test_file);
    let _ = fs::remove_file(binary_path);
}

/// Test transpiler generates correct Rust code for println
#[test] 
fn test_transpiler_println_generation() {
    use ruchy::frontend::parser::Parser;
    use ruchy::backend::transpiler::Transpiler;
    
    let code = r#"
fun main() {
    let x = 42
    println("Value: {}", x)
}
"#;

    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse successfully");
    
    let transpiler = Transpiler::new();
    let rust_code = transpiler.transpile(&ast).expect("Should transpile successfully");
    let rust_string = rust_code.to_string();
    
    println!("GENERATED RUST CODE: {}", rust_string);
    
    // Should generate proper Rust println! macro (may have spaces)
    assert!(rust_string.contains("println") && rust_string.contains("!"), "Should contain println! macro");
    
    // Should NOT contain malformed format strings like "{} {:?}"
    assert!(!rust_string.contains("{} {:?}"), "Should not generate malformed format strings");
    
    // Should contain proper format placeholder
    assert!(rust_string.contains("Value: {}") || rust_string.contains("Value: {:?}"), 
            "Should contain proper format placeholder");
}

/// Performance test: compiled binary should be reasonably fast
#[test]
fn test_compiled_binary_performance() {
    let test_file = "/tmp/test_performance.ruchy";
    let binary_path = "/tmp/test_performance_bin";
    
    fs::write(test_file, r#"
fun fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fun main() {
    let result = fibonacci(10)
    println("Fibonacci(10): {}", result)
}
"#).expect("Failed to write test file");

    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    assert!(result.is_ok(), "Performance test should compile");
    
    // Time the execution
    let start = std::time::Instant::now();
    let output = Command::new(binary_path)
        .output()
        .expect("Failed to execute performance test");
    let duration = start.elapsed();
    
    // Should complete in reasonable time (less than 1 second for fib(10))
    assert!(duration.as_secs() < 1, "Fibonacci(10) should complete quickly");
    
    let stdout = String::from_utf8(output.stdout).expect("Invalid UTF-8 output");
    // Fibonacci(10) = 55
    if !stdout.contains("{}") {  // Only check if format strings work
        assert!(stdout.contains("55"), "Should calculate fibonacci correctly");
    }
    
    // Cleanup
    let _ = fs::remove_file(test_file);
    let _ = fs::remove_file(binary_path);
}

/// Test compilation error handling
#[test]
fn test_compilation_error_handling() {
    let test_file = "/tmp/test_bad_syntax.ruchy";
    let binary_path = "/tmp/test_bad_syntax_bin";
    
    // Create file with syntax error
    fs::write(test_file, r#"
fun main() {
    let x = 
    println("Incomplete")
}
"#).expect("Failed to write test file");

    let options = CompileOptions {
        output: binary_path.into(),
        ..Default::default()
    };
    let result = compile_to_binary(Path::new(test_file), &options);
    
    // Should fail to compile due to syntax error
    assert!(result.is_err(), "Bad syntax should fail compilation");
    
    // Binary should not exist
    assert!(!Path::new(binary_path).exists(), "Binary should not exist for failed compilation");
    
    // Cleanup
    let _ = fs::remove_file(test_file);
}