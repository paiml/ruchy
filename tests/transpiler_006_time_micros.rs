/// TRANSPILER-006: time_micros() builtin function transpilation
///
/// GitHub Issue: #139
/// Blocks: Docker integration examples (fibonacci benchmarks)
///
/// BUG: time_micros() passes through transpilation without being converted
/// IMPACT: error[E0425]: cannot find function `time_micros` in this scope
/// FIX: Transpile to std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() as u64

use ruchy::frontend::parser::Parser;
use ruchy::backend::transpiler::Transpiler;

/// Test 1: Basic time_micros() call
#[test]
fn test_transpiler_006_01_basic_time_micros() {
    let code = r#"
pub fn get_time() -> u64 {
    time_micros()
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    // Should NOT contain raw time_micros() call
    assert!(
        !rust_code.contains("time_micros()"),
        "BUG: Raw time_micros() should not appear in output:\n{}",
        rust_code
    );

    // Should contain Rust stdlib time API
    assert!(
        rust_code.contains("SystemTime") || rust_code.contains("as_micros"),
        "Should use Rust time API:\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_006_01_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_006_01_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: time_micros() fails compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 2: Time difference (benchmarking pattern)
#[test]
fn test_transpiler_006_02_time_difference() {
    let code = r#"
pub fn benchmark() -> u64 {
    let start = time_micros();
    // Some work here
    let end = time_micros();
    end - start
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("time_micros()"),
        "BUG: Raw time_micros() should not appear:\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_006_02_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_006_02_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "Time difference calculation should compile:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 3: Fibonacci benchmark (GitHub Issue #139 - Docker example)
#[test]
fn test_transpiler_006_03_fibonacci_benchmark() {
    let code = r#"
pub fn benchmark() -> u64 {
    let start = time_micros();
    let result = fib(20);
    let end = time_micros();
    end - start
}

fn fib(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fib(n - 1) + fib(n - 2)
    }
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("time_micros()"),
        "BUG: Raw time_micros() in fibonacci benchmark:\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_006_03_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_006_03_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "CRITICAL: Fibonacci benchmark (Issue #139) fails compilation:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}

/// Test 4: Multiple time_micros() calls in same function
#[test]
fn test_transpiler_006_04_multiple_calls() {
    let code = r#"
pub fn multi_benchmark() -> (u64, u64) {
    let t1 = time_micros();
    let work1 = 42 + 42;
    let t2 = time_micros();
    let work2 = 100 * 100;
    let t3 = time_micros();

    let duration1 = t2 - t1;
    let duration2 = t3 - t2;
    (duration1, duration2)
}
"#;

    let ast = Parser::new(code).parse().expect("Parse should succeed");
    let result = Transpiler::new().transpile_to_program(&ast);

    assert!(result.is_ok(), "Should transpile, got: {:?}", result.err());

    let rust_code = result.unwrap().to_string();

    assert!(
        !rust_code.contains("time_micros()"),
        "BUG: Raw time_micros() should not appear:\n{}",
        rust_code
    );

    // Verify rustc compilation
    std::fs::write("/tmp/transpiler_006_04_output.rs", &rust_code)
        .expect("Failed to write test file");

    let rustc_result = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "/tmp/transpiler_006_04_output.rs"])
        .output()
        .expect("Failed to run rustc");

    if !rustc_result.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_result.stderr);
        panic!(
            "Multiple time_micros() calls should compile:\n{}\n\nCode:\n{}",
            stderr, rust_code
        );
    }
}
