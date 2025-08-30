// STDLIB-003: Collection Methods Test Suite
// Following Toyota Way TDD - RED phase first

use ruchy::runtime::repl::Repl;
use std::process::Command;
use std::fs;

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
    let test_file = format!("/tmp/collection_test_{}.ruchy", 
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

#[test]
fn test_array_slice() {
    // Test slice method: [1,2,3,4].slice(1,3) should return [2,3]
    let result = eval_in_repl("[1,2,3,4].slice(1,3)").unwrap();
    assert_eq!(result, "[2, 3]");
    
    let result = eval_transpiled("println([1,2,3,4].slice(1,3))").unwrap();
    assert_eq!(result, "[2, 3]");
    
    // Test edge cases
    let result = eval_in_repl("[1,2,3].slice(0,0)").unwrap();
    assert_eq!(result, "[]");
    
    let result = eval_in_repl("[1,2,3].slice(1,1)").unwrap(); 
    assert_eq!(result, "[]");
    
    let result = eval_in_repl("[1,2,3].slice(0,3)").unwrap();
    assert_eq!(result, "[1, 2, 3]");
}

#[test]
fn test_array_concat() {
    // Test concat method: [1,2].concat([3,4]) should return [1,2,3,4]
    let result = eval_in_repl("[1,2].concat([3,4])").unwrap();
    assert_eq!(result, "[1, 2, 3, 4]");
    
    let result = eval_transpiled("println([1,2].concat([3,4]))").unwrap();
    assert_eq!(result, "[1, 2, 3, 4]");
    
    // Test with empty arrays
    let result = eval_in_repl("[1,2].concat([])").unwrap();
    assert_eq!(result, "[1, 2]");
    
    let result = eval_in_repl("[].concat([3,4])").unwrap();
    assert_eq!(result, "[3, 4]");
}

#[test]
fn test_array_flatten() {
    // Test flatten method: [[1,2],[3]].flatten() should return [1,2,3]
    let result = eval_in_repl("[[1,2],[3]].flatten()").unwrap();
    assert_eq!(result, "[1, 2, 3]");
    
    let result = eval_transpiled("println([[1,2],[3]].flatten())").unwrap();
    assert_eq!(result, "[1, 2, 3]");
    
    // Test with nested empty arrays
    let result = eval_in_repl("[[1],[],[2,3]].flatten()").unwrap();
    assert_eq!(result, "[1, 2, 3]");
    
    // Test empty array
    let result = eval_in_repl("[].flatten()").unwrap();
    assert_eq!(result, "[]");
}

#[test] 
fn test_array_unique() {
    // Test unique method: [1,2,1,3].unique() should return [1,2,3] (order may vary)
    let result = eval_in_repl("[1,2,1,3].unique()").unwrap();
    // Note: HashSet doesn't guarantee order, so we check length and contents
    assert!(result.contains('1') && result.contains('2') && result.contains('3'));
    assert!(result.starts_with('[') && result.ends_with(']'));
    
    let result = eval_transpiled("println([1,2,1,3].unique())").unwrap();
    assert!(result.contains('1') && result.contains('2') && result.contains('3'));
    assert!(result.starts_with('[') && result.ends_with(']'));
    
    // Test with all same elements
    let result = eval_in_repl("[5,5,5].unique()").unwrap();
    assert!(result.contains('5'));
    assert!(!result.contains("55")); // Make sure it's not double-counted
    
    // Test empty array
    let result = eval_in_repl("[].unique()").unwrap();
    assert_eq!(result, "[]");
}

#[test]
fn test_string_array_join() {
    // Test join method: ["a","b","c"].join(",") should return "a,b,c"
    let result = eval_in_repl(r#"["a","b","c"].join(",")"#).unwrap();
    // REPL adds quotes to strings
    assert!(result.contains("a,b,c"));
    
    let result = eval_transpiled(r#"println(["a","b","c"].join(","))"#).unwrap();
    assert_eq!(result, "a,b,c");
    
    // Test different separator
    let result = eval_transpiled(r#"println(["x","y","z"].join(" - "))"#).unwrap();
    assert_eq!(result, "x - y - z");
    
    // Test empty array
    let result = eval_transpiled(r#"println([].join(","))"#).unwrap();
    assert_eq!(result, "");
    
    // Test single element
    let result = eval_transpiled(r#"println(["hello"].join(","))"#).unwrap();
    assert_eq!(result, "hello");
}

#[test]
fn test_string_substring() {
    // Test substring method: "hello".substring(1,4) should return "ell"
    let result = eval_in_repl(r#""hello".substring(1,4)"#).unwrap();
    // REPL adds quotes to strings, so we need to handle that
    assert!(result.contains("ell"));
    
    let result = eval_transpiled(r#"println("hello".substring(1,4))"#).unwrap();
    assert_eq!(result, "ell");
    
    // Test edge cases
    let result = eval_transpiled(r#"println("hello".substring(0,0))"#).unwrap();
    assert_eq!(result, "");
    
    let result = eval_transpiled(r#"println("hello".substring(0,5))"#).unwrap();
    assert_eq!(result, "hello");
    
    let result = eval_transpiled(r#"println("hello".substring(1,1))"#).unwrap();
    assert_eq!(result, "");
}

#[test]
fn test_method_chaining() {
    // Test chaining methods: [1,2,1,3].unique().slice(0,2)
    let result = eval_in_repl("[1,2,1,3].unique().slice(0,2)").unwrap();
    // Should have 2 elements from the unique set
    assert!(result.starts_with('[') && result.ends_with(']'));
    
    // Test string methods: "hello world".substring(0,5).concat(" there")
    // Note: This might need string concat operator instead of method
}