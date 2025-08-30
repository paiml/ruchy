// STDLIB-002: Advanced Math Functions Test Suite
// Following Toyota Way TDD - RED phase first

use ruchy::runtime::repl::Repl;
use std::process::Command;
use std::fs;
use std::f64::consts::{PI, E};

// Helper to test in REPL
fn eval_in_repl(code: &str) -> Result<String, String> {
    let mut repl = Repl::new()
        .map_err(|e| format!("Failed to create REPL: {e:?}"))?;
    
    let result = repl.eval(code)
        .map_err(|e| format!("Eval error: {e:?}"))?;
    
    Ok(result)
}

// Helper to test transpiled code with unique filenames
fn eval_transpiled(code: &str) -> Result<String, String> {
    let test_file = format!("/tmp/math_test_{}.ruchy", 
        std::process::id());
    fs::write(&test_file, code)
        .map_err(|e| format!("Failed to write test file: {e}"))?;
    
    let output = Command::new("./target/release/ruchy")
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run file: {e}"))?;
    
    // Clean up
    let _ = fs::remove_file(&test_file);
    
    if !output.status.success() {
        return Err(format!("Execution failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

// Parse float from result string
fn parse_float(result: &str) -> f64 {
    result.parse().unwrap_or(f64::NAN)
}

#[test]
fn test_sin_function() {
    // Test sin(0) = 0
    let result = parse_float(&eval_in_repl("sin(0.0)").unwrap());
    assert!((result - 0.0).abs() < 0.0001, "sin(0) should be 0, got {result}");
    
    let result = parse_float(&eval_transpiled("println(sin(0.0))").unwrap());
    assert!((result - 0.0).abs() < 0.0001, "sin(0) should be 0, got {result}");
    
    // Test sin(π/2) = 1
    let pi_2 = PI / 2.0;
    let code = format!("sin({pi_2})");
    let result = parse_float(&eval_in_repl(&code).unwrap());
    assert!((result - 1.0).abs() < 0.0001, "sin(π/2) should be 1, got {result}");
    
    let code = format!("println(sin({pi_2}))");
    let result = parse_float(&eval_transpiled(&code).unwrap());
    assert!((result - 1.0).abs() < 0.0001, "sin(π/2) should be 1, got {result}");
}

#[test]
fn test_cos_function() {
    // Test cos(0) = 1
    let result = parse_float(&eval_in_repl("cos(0.0)").unwrap());
    assert!((result - 1.0).abs() < 0.0001, "cos(0) should be 1, got {result}");
    
    let result = parse_float(&eval_transpiled("println(cos(0.0))").unwrap());
    assert!((result - 1.0).abs() < 0.0001, "cos(0) should be 1, got {result}");
    
    // Test cos(π) = -1
    let code = format!("cos({PI})");
    let result = parse_float(&eval_in_repl(&code).unwrap());
    assert!((result - (-1.0)).abs() < 0.0001, "cos(π) should be -1, got {result}");
    
    let code = format!("println(cos({PI}))");
    let result = parse_float(&eval_transpiled(&code).unwrap());
    assert!((result - (-1.0)).abs() < 0.0001, "cos(π) should be -1, got {result}");
}

#[test]
fn test_tan_function() {
    // Test tan(0) = 0
    let result = parse_float(&eval_in_repl("tan(0.0)").unwrap());
    assert!((result - 0.0).abs() < 0.0001, "tan(0) should be 0, got {result}");
    
    let result = parse_float(&eval_transpiled("println(tan(0.0))").unwrap());
    assert!((result - 0.0).abs() < 0.0001, "tan(0) should be 0, got {result}");
    
    // Test tan(π/4) = 1
    let pi_4 = PI / 4.0;
    let code = format!("tan({pi_4})");
    let result = parse_float(&eval_in_repl(&code).unwrap());
    assert!((result - 1.0).abs() < 0.0001, "tan(π/4) should be 1, got {result}");
    
    let code = format!("println(tan({pi_4}))");
    let result = parse_float(&eval_transpiled(&code).unwrap());
    assert!((result - 1.0).abs() < 0.0001, "tan(π/4) should be 1, got {result}");
}

#[test]
fn test_log_function() {
    // Test log(e) = 1
    let code = format!("log({E})");
    let result = parse_float(&eval_in_repl(&code).unwrap());
    assert!((result - 1.0).abs() < 0.0001, "log(e) should be 1, got {result}");
    
    let code = format!("println(log({E}))");
    let result = parse_float(&eval_transpiled(&code).unwrap());
    assert!((result - 1.0).abs() < 0.0001, "log(e) should be 1, got {result}");
    
    // Test log(1) = 0
    let result = parse_float(&eval_in_repl("log(1.0)").unwrap());
    assert!((result - 0.0).abs() < 0.0001, "log(1) should be 0, got {result}");
    
    let result = parse_float(&eval_transpiled("println(log(1.0))").unwrap());
    assert!((result - 0.0).abs() < 0.0001, "log(1) should be 0, got {result}");
}

#[test]
fn test_log10_function() {
    // Test log10(10) = 1
    let result = parse_float(&eval_in_repl("log10(10.0)").unwrap());
    assert!((result - 1.0).abs() < 0.0001, "log10(10) should be 1, got {result}");
    
    let result = parse_float(&eval_transpiled("println(log10(10.0))").unwrap());
    assert!((result - 1.0).abs() < 0.0001, "log10(10) should be 1, got {result}");
    
    // Test log10(100) = 2
    let result = parse_float(&eval_in_repl("log10(100.0)").unwrap());
    assert!((result - 2.0).abs() < 0.0001, "log10(100) should be 2, got {result}");
    
    let result = parse_float(&eval_transpiled("println(log10(100.0))").unwrap());
    assert!((result - 2.0).abs() < 0.0001, "log10(100) should be 2, got {result}");
}

#[test]
fn test_random_function() {
    // Test that random() returns value between 0 and 1
    for _ in 0..10 {
        let result = parse_float(&eval_in_repl("random()").unwrap());
        assert!((0.0..=1.0).contains(&result), 
            "random() should return value between 0 and 1, got {result}");
    }
    
    // Test transpiled version
    let code = r"
        println(random())
        println(random())
        println(random())
        println(random())
        println(random())
    ";
    let output = eval_transpiled(code).unwrap();
    for line in output.lines() {
        if !line.is_empty() {
            let value = parse_float(line);
            assert!((0.0..=1.0).contains(&value), 
                "random() should return value between 0 and 1, got {value}");
        }
    }
}

#[test]
fn test_math_composition() {
    // Test composition of functions: sin²(x) + cos²(x) = 1
    let x = 0.7;
    let code = format!("sin({x}) * sin({x}) + cos({x}) * cos({x})");
    let result = parse_float(&eval_in_repl(&code).unwrap());
    assert!((result - 1.0).abs() < 0.0001, 
        "sin²(x) + cos²(x) should equal 1, got {result}");
    
    let code = format!("println(sin({x}) * sin({x}) + cos({x}) * cos({x}))");
    let result = parse_float(&eval_transpiled(&code).unwrap());
    assert!((result - 1.0).abs() < 0.0001, 
        "sin²(x) + cos²(x) should equal 1, got {result}");
}