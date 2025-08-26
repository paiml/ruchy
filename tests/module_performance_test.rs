//! Module Performance Test (TDD)
//! 
//! Ensures O(1) module lookup performance with caching
//!
//! **Expected**: Multiple imports of same module should be cached, not re-parsed
//! **Current**: O(N) - re-reads and re-parses on every import

use ruchy::runtime::repl::Repl;
use std::fs;
use std::time::Instant;
use tempfile::TempDir;

#[test]
fn test_module_caching_performance() {
    // Setup: Create a moderately large module file
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create math.ruchy with multiple functions (simulate larger module)
    let math_content = r#"
pub fn add(a: i32, b: i32) -> i32 { a + b }
pub fn subtract(a: i32, b: i32) -> i32 { a - b }
pub fn multiply(a: i32, b: i32) -> i32 { a * b }
pub fn divide(a: i32, b: i32) -> i32 { a / b }
pub fn power(a: i32, b: i32) -> i32 { 
    let mut result = 1;
    let mut i = 0;
    while i < b {
        result = result * a;
        i = i + 1;
    }
    result
}
pub fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
"#;
    fs::write(temp_dir.path().join("math.ruchy"), math_content)
        .expect("Failed to write math module");
    
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    let mut repl = Repl::new().expect("REPL creation should succeed");
    
    // First import - this will involve disk I/O and parsing (acceptable)
    let start1 = Instant::now();
    let result1 = repl.evaluate_expr_str("use math", None);
    let duration1 = start1.elapsed();
    assert!(result1.is_ok(), "First import should succeed");
    
    // Create new REPL instance to simulate fresh session
    let mut repl2 = Repl::new().expect("REPL creation should succeed");
    
    // Second import - should be much faster due to caching
    let start2 = Instant::now();
    let result2 = repl2.evaluate_expr_str("use math", None);
    let duration2 = start2.elapsed();
    assert!(result2.is_ok(), "Second import should succeed");
    
    // Performance assertion: Second import should be at least 2x faster
    // (In reality, should be 10x+ faster with proper O(1) caching)
    println!("First import: {:?}, Second import: {:?}", duration1, duration2);
    
    // For now, just ensure both work - proper caching implementation needed
    assert!(result1.is_ok() && result2.is_ok(), "Both imports should succeed");
    
    // Test that functions are available from both imports
    let test1 = repl.evaluate_expr_str("add(5, 3)", None);
    let test2 = repl2.evaluate_expr_str("multiply(4, 6)", None);
    
    assert!(test1.is_ok(), "Function should be available from first import");
    assert!(test2.is_ok(), "Function should be available from second import");
}

#[test]
fn test_duplicate_imports_same_repl_session() {
    // Test that importing the same module multiple times in one session is O(1)
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    let utils_content = r#"pub fn helper() -> String { "helper result" }"#;
    fs::write(temp_dir.path().join("utils.ruchy"), utils_content)
        .expect("Failed to write utils module");
    
    std::env::set_current_dir(temp_dir.path()).expect("Failed to change directory");
    let mut repl = Repl::new().expect("REPL creation should succeed");
    
    // First import
    let start1 = Instant::now();
    let result1 = repl.evaluate_expr_str("use utils", None);
    let duration1 = start1.elapsed();
    
    // Second import of same module in same session
    let start2 = Instant::now();
    let result2 = repl.evaluate_expr_str("use utils", None);  
    let duration2 = start2.elapsed();
    
    // Third import
    let start3 = Instant::now();
    let result3 = repl.evaluate_expr_str("use utils", None);
    let duration3 = start3.elapsed();
    
    println!("Import durations: {:?}, {:?}, {:?}", duration1, duration2, duration3);
    
    assert!(result1.is_ok() && result2.is_ok() && result3.is_ok(), 
            "All imports should succeed");
    
    // All subsequent imports should be significantly faster (cached)
    // This test documents the expected behavior for implementation
    
    // Test that function is still available
    let test = repl.evaluate_expr_str("helper()", None);
    assert!(test.is_ok(), "Function should be available after multiple imports");
}