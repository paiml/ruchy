// Regression Database - Every bug gets a permanent test to prevent re-occurrence
// Toyota Way: Once fixed, it must never break again

use ruchy::runtime::Repl;
use std::process::Command;
use std::fs;

/// Regression tests for specific bugs that were found and fixed
/// Each test documents the original issue and prevents re-occurrence

#[test]
fn regression_while_loop_off_by_one() {
    // BUG REPORT: While loop printed extra iteration (0,1,2,3 instead of 0,1,2)  
    // ROOT CAUSE: While loop returned body value instead of Unit
    // FIX: Always return Value::Unit from while loops
    // DATE: 2024-12-XX
    
    let mut repl = Repl::new().unwrap();
    
    // Test 1: Exact original failing case (now using var for mutable bindings)
    let result = repl.eval("var i = 0; var count = 0; while i < 3 { count = count + 1; i = i + 1 }; count").unwrap();
    assert_eq!(result, "3", "While loop should execute exactly 3 times");
    
    // Test 2: While loop should return Unit, not body value
    let result = repl.eval("var i = 0; while i < 1 { i = i + 1; 42 }").unwrap(); 
    assert_eq!(result, "()", "While loop should return Unit, not 42");
    
    // Test 3: Multiple iterations with different body values
    let result = repl.eval("var i = 0; while i < 2 { i = i + 1; i * 10 }").unwrap();
    assert_eq!(result, "()", "While loop should return Unit, not last body value");
}

#[test] 
fn regression_object_items_transpilation() {
    // BUG REPORT: obj.items() worked in REPL but failed when transpiled to files
    // ROOT CAUSE: Transpiler converted items() to iter() which has wrong signature  
    // FIX: Convert items() to iter().map(|(k,v)| (k.clone(), v.clone()))
    // DATE: 2024-12-XX
    
    // Test via file execution (this used to fail)
    let code = r#"let obj = {"test": 123}; for key, value in obj.items() { println(key) }"#;
    fs::write("/tmp/regression_items_test.ruchy", code).unwrap();
    
    let output = Command::new("./target/release/ruchy")
        .arg("/tmp/regression_items_test.ruchy")
        .output()
        .unwrap();
    
    assert!(output.status.success(), 
        "obj.items() file execution should work, stderr: {}", 
        String::from_utf8_lossy(&output.stderr));
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("test"), 
        "Should print object keys, got: {stdout}");
}

#[test]
fn regression_test_compilation_errors() {
    // BUG REPORT: Raw string syntax in tests caused compilation failures
    // ROOT CAUSE: Using r"..." instead of r#"..."# when strings contain quotes
    // FIX: Replace all r"..." with r#"..."# in test files
    // DATE: 2024-12-XX
    
    // This should compile without errors now
    let test_code = r#"println("Hello \"World\"")"#;
    
    let mut repl = Repl::new().unwrap();
    let result = repl.eval(test_code);
    assert!(result.is_ok(), "Raw string test code should compile: {result:?}");
}

#[test]
fn regression_make_lint_underscore_binding() {
    // BUG REPORT: make lint failed with "used underscore-prefixed binding" 
    // ROOT CAUSE: Variable named _ast was used instead of ast
    // FIX: Remove underscore prefix when variable is actually used
    // DATE: 2024-12-XX
    
    // Skip in CI/pre-commit to avoid timeout - lint is checked separately
    if std::env::var("CI").is_ok() || std::env::var("PRE_COMMIT").is_ok() {
        println!("Skipping make lint test in CI/pre-commit");
        return;
    }
    
    // Verify lint passes now with timeout
    let output = Command::new("timeout")
        .arg("30")
        .arg("make")
        .arg("lint")
        .output()
        .unwrap();
    
    assert!(output.status.success(), 
        "make lint should pass, stderr: {}", 
        String::from_utf8_lossy(&output.stderr));
}

#[test]
fn regression_coverage_script_flags() {
    // BUG REPORT: Coverage script failed with "--html may not be used together with --lcov"
    // ROOT CAUSE: cargo-llvm-cov called with conflicting flags simultaneously  
    // FIX: Split into separate HTML and LCOV generation steps
    // DATE: 2024-12-XX
    
    // Skip in CI/pre-commit to avoid long compile times
    if std::env::var("CI").is_ok() || std::env::var("PRE_COMMIT").is_ok() {
        println!("Skipping coverage test in CI/pre-commit");
        return;
    }
    
    // Test that coverage commands work individually with timeout
    let html_test = Command::new("timeout")
        .arg("30")
        .arg("cargo")
        .args(["llvm-cov", "--lib", "--html", "--output-dir", "/tmp/test_coverage"])
        .output()
        .unwrap();
    
    // Should not fail with flag conflict error
    let stderr = String::from_utf8_lossy(&html_test.stderr);
    assert!(!stderr.contains("may not be used together"), 
        "Coverage HTML generation should work: {stderr}");
}

/// Future regression test template - copy this for new bugs
#[test]
fn regression_template_future_bug() {
    // BUG REPORT: [Describe what broke]
    // ROOT CAUSE: [Technical reason it broke]  
    // FIX: [What was changed to fix it]
    // DATE: [When it was fixed]
    
    // Test that demonstrates the bug is fixed
    // This should fail before the fix and pass after
    
    // Example:
    // let mut repl = Repl::new().unwrap();
    // let result = repl.eval("code that used to break").unwrap();
    // assert_eq!(result.to_string(), "expected_result");
}

/// Performance regression tests
#[test]
fn regression_performance_not_degraded() {
    // Ensure critical operations stay fast
    use std::time::Instant;
    
    let mut repl = Repl::new().unwrap();
    
    // Simple arithmetic should be very fast
    let start = Instant::now();
    repl.eval("2 + 2").unwrap();
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 100, 
        "Simple arithmetic took too long: {}ms", duration.as_millis());
    
    // Function calls should be reasonable
    let start = Instant::now();
    repl.eval("fn f(x) { x + 1 }; f(42)").unwrap();
    let duration = start.elapsed();
    
    assert!(duration.as_millis() < 200,
        "Function call took too long: {}ms", duration.as_millis());
}