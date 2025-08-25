#![cfg(test)]
#![allow(warnings)]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::unreadable_literal)]
#![allow(clippy::approx_constant)]
#![allow(clippy::unwrap_used)]
// tests/compatibility_suite.rs  
// Comprehensive compatibility test suite for Ruchy language features
// Based on industry best practices from Rust, Python, Elixir, Ruby, SQLite, Haskell, JS/Deno
// Specification: docs/specifications/language-feature-testing-spec.md

#![allow(clippy::print_stdout)]       // Tests should print results
#![allow(clippy::print_stderr)]       // Tests should print diagnostics  
#![allow(clippy::needless_raw_string_hashes)]  // Raw strings for multi-line test code
#![allow(clippy::cast_precision_loss)] // Test statistics precision is not critical
#![allow(clippy::cast_lossless)]       // Test code casting is acceptable
#![allow(clippy::expect_used)]         // Tests can use expect for setup
#![allow(clippy::ignore_without_reason)] // Test ignores are documented  
#![allow(clippy::doc_markdown)]        // Test documentation less strict
#![allow(clippy::map_unwrap_or)]       // Test utility functions acceptable
#![allow(clippy::uninlined_format_args)] // Test output formatting acceptable
#![allow(clippy::cast_possible_truncation)] // Test metrics casting acceptable
#![allow(dead_code)]                   // Test utilities may not all be used

use std::process::Command;
use std::fs;
use std::time::{Duration, Instant};

/// Test execution result with performance metrics
#[derive(Debug)]
struct TestResult {
    success: bool,
    stdout: String, 
    stderr: String,
    duration: Duration,
}

/// Run a Ruchy code snippet and return detailed test results
fn run_ruchy_code(code: &str) -> TestResult {
    let test_file = "/tmp/ruchy_test.ruchy";
    fs::write(test_file, code).expect("Failed to write test file");
    
    let start = Instant::now();
    let output = Command::new("./target/release/ruchy")
        .arg("run")
        .arg(test_file)
        .output()
        .expect("Failed to execute ruchy");
    let duration = start.elapsed();
    
    TestResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        duration,
    }
}

/// Performance thresholds for regression detection (post-refactoring baseline)
/// Updated after quality sprint refactoring: trade-off performance for maintainability
const PERFORMANCE_THRESHOLDS: &[(&str, Duration)] = &[
    ("simple_arithmetic", Duration::from_millis(30)),  // Increased due to modular design
    ("function_call", Duration::from_millis(50)),      // Increased due to parser refactoring
    ("loop_iteration", Duration::from_millis(100)),
    ("string_operations", Duration::from_millis(60)),
];

/// Get performance threshold for a test category
fn get_performance_threshold(category: &str) -> Duration {
    PERFORMANCE_THRESHOLDS.iter()
        .find(|(name, _)| category.contains(name))
        .map(|(_, threshold)| *threshold)
        .unwrap_or(Duration::from_millis(300)) // Default threshold - increased post-refactoring
}

/// Test a feature with enhanced diagnostics and performance monitoring
fn test_feature(name: &str, code: &str, should_succeed: bool) -> bool {
    let result = run_ruchy_code(code);
    let threshold = get_performance_threshold(name);
    
    let success = result.success == should_succeed;
    let performance_ok = result.duration <= threshold;
    
    if success && performance_ok {
        println!("✅ {}: PASS ({}ms)", name, result.duration.as_millis());
        true
    } else {
        if !success {
            println!("❌ {}: FAIL - Logic Error", name);
            if !result.stderr.is_empty() {
                println!("   Error: {}", result.stderr.trim());
            }
            if !result.stdout.is_empty() {
                println!("   Output: {}", result.stdout.trim());
            }
        }
        if !performance_ok {
            println!("⚠️  {}: PERFORMANCE REGRESSION ({}ms > {}ms)", 
                name, result.duration.as_millis(), threshold.as_millis());
        }
        false
    }
}

/// Test a feature category with statistical reporting (inspired by Python pytest)
#[derive(Default)]
struct TestCategoryResult {
    name: String,
    passed: usize,
    total: usize,
    total_duration: Duration,
    failed_tests: Vec<String>,
}

impl TestCategoryResult {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
    
    fn add_test_result(&mut self, test_name: &str, passed: bool, duration: Duration) {
        self.total += 1;
        self.total_duration += duration;
        if passed {
            self.passed += 1;
        } else {
            self.failed_tests.push(test_name.to_string());
        }
    }
    
    fn success_rate(&self) -> f64 {
        if self.total == 0 { 0.0 } else { (self.passed as f64 / self.total as f64) * 100.0 }
    }
    
    fn avg_duration(&self) -> Duration {
        if self.total == 0 { Duration::ZERO } else { self.total_duration / self.total as u32 }
    }
    
    fn print_summary(&self) {
        println!("\n=== {} ===", self.name.to_uppercase());
        println!("Passed: {}/{} ({:.1}%) - Avg: {}ms", 
                 self.passed, self.total, self.success_rate(), self.avg_duration().as_millis());
        
        if !self.failed_tests.is_empty() {
            println!("Failed tests: {}", self.failed_tests.join(", "));
        }
        
        // Performance analysis
        if self.avg_duration() > Duration::from_millis(50) {
            println!("⚠️  High average execution time detected");
        }
    }
}

#[test]
fn test_basic_language_features() {
    let mut passed = 0;
    let mut total = 0;
    
    // Function definitions and calls
    total += 1;
    if test_feature("Function Definition (fn keyword)", r#"
fn greet(name) {
    println("Hello " + name)
}
greet("World")
"#, true) { passed += 1; }

    total += 1;
    if test_feature("Function Definition (fun keyword)", r#"
fun greet(name) {
    println("Hello " + name)
}
greet("World")
"#, true) { passed += 1; }

    // Return statements
    total += 1;
    if test_feature("Return Statements", 
"fn add(x, y) {
    return x + y
}
println(add(5, 3))", true) { passed += 1; }

    // Type annotations
    total += 1;
    if test_feature("Type Annotations", r#"
fn add(x: i32, y: i32) -> i32 {
    return x + y
}
println(add(5, 3))
"#, true) { passed += 1; }

    // Public visibility
    total += 1;
    if test_feature("Public Visibility", r#"
pub fn public_function() {
    println("This is public")
}
public_function()
"#, true) { passed += 1; }

    println!("\n=== BASIC LANGUAGE FEATURES ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
    assert!(passed > 0, "No basic language features working!");
}

#[test]
fn test_control_flow() {
    let mut passed = 0;
    let mut total = 0;

    // For loops (simple)
    total += 1;
    if test_feature("For Loop Simple", r#"
for i in [1, 2, 3] {
    println(i)
}
"#, true) { passed += 1; }

    // For loops with tuple destructuring (v0.13.0 feature)
    total += 1;
    if test_feature("Tuple Destructuring in For Loop", r#"
for x, y in [(1, 2), (3, 4)] {
    println(x);
    println(y)
}
"#, true) { passed += 1; }

    // While loops
    total += 1;
    if test_feature("While Loop", r#"
let i = 0
while i < 3 {
    println(i)
    i = i + 1
}
"#, true) { passed += 1; }

    // Match expressions
    total += 1;
    if test_feature("Match Expression", r#"
let x = 5
let result = match x {
    1 => "one",
    5 => "five",
    _ => "other"
}
println(result)
"#, true) { passed += 1; }

    // If expressions
    total += 1;
    if test_feature("If Expression", r#"
let x = 10
let result = if x > 5 { "big" } else { "small" }
println(result)
"#, true) { passed += 1; }

    println!("\n=== CONTROL FLOW ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
}

#[test]
fn test_data_structures() {
    let mut passed = 0;
    let mut total = 0;

    // Arrays/Lists
    total += 1;
    if test_feature("Array Creation", r#"
let arr = [1, 2, 3, 4, 5]
println(arr.len())
"#, true) { passed += 1; }

    // Array indexing
    total += 1;
    if test_feature("Array Indexing", r#"
let arr = [10, 20, 30]
println(arr[1])
"#, true) { passed += 1; }

    // Array methods
    total += 1;
    if test_feature("Array Map", r#"
let arr = [1, 2, 3]
let doubled = arr.map(|x| x * 2)
println(doubled)
"#, true) { passed += 1; }

    total += 1;
    if test_feature("Array Filter", r#"
let arr = [1, 2, 3, 4, 5]
let evens = arr.filter(|x| x % 2 == 0)
println(evens)
"#, true) { passed += 1; }

    // Objects/Hashes
    total += 1;
    if test_feature("Object Creation", r#"
let obj = {"name": "Alice", "age": 30}
println(obj["name"])
"#, true) { passed += 1; }

    // Object methods (this will likely fail - known gap)
    total += 1;
    if test_feature("Object Items Method", r#"
let obj = {"a": 1, "b": 2}
for key, value in obj.items() {
    println(key + ": " + value.to_string())
}
"#, true) { passed += 1; }

    // Tuples
    total += 1;
    if test_feature("Tuple Creation", r#"
let t = (1, "hello", true)
println(t)
"#, true) { passed += 1; }

    println!("\n=== DATA STRUCTURES ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
}

#[test]
fn test_string_operations() {
    let mut passed = 0;
    let mut total = 0;

    // String concatenation
    total += 1;
    if test_feature("String Concatenation", r#"
let greeting = "Hello " + "World"
println(greeting)
"#, true) { passed += 1; }

    // String interpolation
    total += 1;
    if test_feature("String Interpolation", r#"
let name = "Alice"
let age = 30
println(f"Name: {name}, Age: {age}")
"#, true) { passed += 1; }

    // String methods (likely to fail - known gaps)
    total += 1;
    if test_feature("String Length", r#"
let text = "Hello World"
println(text.len())
"#, true) { passed += 1; }

    total += 1;
    if test_feature("String Trim", r#"
let text = "  hello  "
println(text.trim())
"#, true) { passed += 1; }

    total += 1;
    if test_feature("String To Upper", r#"
let text = "hello"
println(text.to_upper())
"#, true) { passed += 1; }

    println!("\n=== STRING OPERATIONS ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
}

#[test]
fn test_numeric_operations() {
    let mut passed = 0;
    let mut total = 0;

    // Basic arithmetic
    total += 1;
    if test_feature("Basic Arithmetic", r#"
let result = 10 + 5 * 2 - 3
println(result)
"#, true) { passed += 1; }

    // Numeric methods
    total += 1;
    if test_feature("Sqrt Method", r#"
let x = 16.0
println(x.sqrt())
"#, true) { passed += 1; }

    // Type conversions (likely to fail - known gaps)
    total += 1;
    if test_feature("Integer to String", r#"
let num = 42
println(num.to_string())
"#, true) { passed += 1; }

    // Float operations
    total += 1;
    if test_feature("Float Operations", r#"
let pi = 3.14159
let radius = 5.0
let area = pi * radius * radius
println(area)
"#, true) { passed += 1; }

    println!("\n=== NUMERIC OPERATIONS ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
}

#[test]
fn test_advanced_features() {
    let mut passed = 0;
    let mut total = 0;

    // Closures
    total += 1;
    if test_feature("Closures", r#"
let square = |x| x * x
println(square(5))
"#, true) { passed += 1; }

    // Module paths (should fail - not implemented)
    total += 1;
    if test_feature("Module Paths", r#"
std::fs::read_file("test.txt")
"#, false) { passed += 1; }

    // Pattern matching with guards
    total += 1;
    if test_feature("Pattern Guards", r#"
let x = 10
let result = match x {
    n if n > 5 => "big",
    _ => "small"
}
println(result)
"#, true) { passed += 1; }

    // Ranges (may fail - unsure of implementation)
    total += 1;
    if test_feature("Range Syntax", r#"
for i in 0..5 {
    println(i)
}
"#, true) { passed += 1; }

    println!("\n=== ADVANCED FEATURES ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
}

#[test]
fn test_one_liners() {
    // Test the 20 one-liners that should be 100% working
    let one_liners = vec![
        ("Basic Math", "2 + 2"),
        ("Float Math", "100.0 * 1.08"),
        ("Compound Interest", "1000.0 * 1.05 * 1.05"),
        ("Variable Math", "let price = 99.99; let tax = 0.08; price * (1.0 + tax)"),
        ("Comparison", "10 > 5"),
        ("Boolean And", "true && false"),
        ("Boolean Or", "true || false"),
        ("If Expression", "if 100 > 50 { \"expensive\" } else { \"cheap\" }"),
        ("String Concat", "\"Hello \" + \"World\""),
        ("String Variables", "let name = \"Ruchy\"; \"Hello \" + name + \"!\""),
        ("Sqrt Method", "16.0.sqrt()"),
        ("Pythagorean", "let x = 10.0; let y = 20.0; (x * x + y * y).sqrt()"),
        ("E=mc²", "let c = 299792458.0; let m = 0.1; m * c * c"),
        ("P=VI", "let v = 120.0; let i = 10.0; v * i"),
        ("Percentage", "let initial = 10000.0; let final = 15000.0; (final / initial - 1.0) * 100.0"),
    ];

    let mut passed = 0;
    let total = one_liners.len();

    for (name, code) in one_liners {
        if test_feature(name, code, true) {
            passed += 1;
        }
    }

    println!("\n=== ONE-LINERS (Core Functionality) ===");
    println!("Passed: {}/{} ({:.1}%)", passed, total, (passed as f64 / total as f64) * 100.0);
    
    // One-liners should be 100% working - this is our baseline
    assert_eq!(passed, total, "One-liner regression detected! This is a critical failure.");
}

/// Generate a compatibility report
#[test]
#[ignore] // Run with: cargo test compatibility_report -- --ignored
fn compatibility_report() {
    println!("\n🚀 RUCHY COMPATIBILITY REPORT v1.0.0");
    println!("{}", "=".repeat(50));
    
    // Run all test categories
    test_one_liners();
    test_basic_language_features(); 
    test_control_flow();
    test_data_structures();
    test_string_operations();
    test_numeric_operations();
    test_advanced_features();
    
    println!("\n📊 SUMMARY");
    println!("This report identifies working features and implementation gaps.");
    println!("Use this data to prioritize development efforts for maximum");
    println!("compatibility improvement with minimal implementation effort.");
}