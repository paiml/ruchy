// STDLIB Generic Types Test Suite (BUG-006)
// Following Toyota Way TDD - testing existing support

use ruchy::runtime::repl::Repl;
use std::process::Command;
use std::fs;

// Helper to test in REPL
fn eval_in_repl(code: &str) -> Result<String, String> {
    let mut repl = Repl::new()
        .map_err(|e| format!("Failed to create REPL: {:?}", e))?;
    
    let result = repl.eval(code)
        .map_err(|e| format!("Eval error: {:?}", e))?;
    
    // Remove quotes if present (REPL string formatting)
    if result.starts_with('"') && result.ends_with('"') && result.len() >= 2 {
        Ok(result[1..result.len()-1].to_string())
    } else {
        Ok(result)
    }
}

// Helper to test transpiled code with unique filenames
fn eval_transpiled(code: &str) -> Result<String, String> {
    let test_file = format!("/tmp/generic_types_test_{}.ruchy", 
        std::process::id());
    fs::write(&test_file, code)
        .map_err(|e| format!("Failed to write test file: {}", e))?;
    
    let output = Command::new("./target/release/ruchy")
        .arg(&test_file)
        .output()
        .map_err(|e| format!("Failed to run file: {}", e))?;
    
    // Clean up
    let _ = fs::remove_file(&test_file);
    
    if !output.status.success() {
        return Err(format!("Execution failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[test]
fn test_option_some() {
    // Test Some constructor
    let result = eval_in_repl("Some(42)").unwrap();
    assert!(result.contains("Some") && result.contains("42"), 
        "Some should contain value: {}", result);
    
    // Test transpiled version
    let code = "println(Some(42))";
    let result = eval_transpiled(code).unwrap();
    assert!(result.contains("Some") && result.contains("42"), 
        "Some should work in transpiler: {}", result);
}

#[test]
fn test_option_none() {
    // Test None identifier
    let result = eval_in_repl("None").unwrap();
    assert!(result.contains("None"), "None should be available: {}", result);
    
    // Test transpiled version
    let code = "println(None)";
    let result = eval_transpiled(code).unwrap();
    assert!(result.contains("None"), "None should work in transpiler: {}", result);
}

#[test]
fn test_vec_generic_annotations() {
    // Test Vec with type annotations work
    let code = r#"let v: Vec<i32> = [1, 2, 3]
v"#;
    let result = eval_in_repl(code);
    assert!(result.is_ok(), "Vec<i32> annotation should work: {:?}", result);
    
    let vec_str = result.unwrap();
    assert!(vec_str.contains("1") && vec_str.contains("2") && vec_str.contains("3"), 
        "Vec should contain elements: {}", vec_str);
    
    // Test transpiled version
    let code = r#"let v: Vec<i32> = [1, 2, 3]
println(v)"#;
    let result = eval_transpiled(code).unwrap();
    assert!(result.contains("1") && result.contains("2") && result.contains("3"), 
        "Vec should work in transpiler: {}", result);
}

#[test]
fn test_hashmap_generic_annotations() {
    // Test HashMap with type annotations work
    let code = r#"let m: HashMap<String, i32> = {"key": 42}
m"#;
    let result = eval_in_repl(code);
    assert!(result.is_ok(), "HashMap<String, i32> annotation should work: {:?}", result);
    
    let map_str = result.unwrap();
    assert!(map_str.contains("key") && map_str.contains("42"), 
        "HashMap should contain key-value: {}", map_str);
    
    // Test transpiled version  
    let code = r#"let m: HashMap<String, i32> = {"key": 42}
println(m)"#;
    let result = eval_transpiled(code).unwrap();
    assert!(result.contains("key") && result.contains("42"), 
        "HashMap should work in transpiler: {}", result);
}

#[test]
fn test_option_pattern_matching() {
    // Test Option in match expressions
    let code = r#"
let opt = Some(42)
match opt {
    Some(x) => x * 2,
    None => 0
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok(), "Option pattern matching should work: {:?}", result);
    assert_eq!(result.unwrap(), "84", "Some(42) * 2 should be 84");
    
    // Test None case
    let code = r#"
let opt = None
match opt {
    Some(x) => x * 2,  
    None => 999
}
"#;
    
    let result = eval_in_repl(code).unwrap();
    assert_eq!(result, "999", "None should match None case");
}

#[test]
fn test_nested_generics() {
    // Test nested generic types
    let code = r#"let nested: Vec<Option<i32>> = [Some(1), None, Some(3)]
nested"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok(), "Nested generics should work: {:?}", result);
    
    let nested_str = result.unwrap();
    assert!(nested_str.contains("Some") && nested_str.contains("None"), 
        "Nested generics should contain Option types: {}", nested_str);
}

#[test]
fn test_generic_functions_type_annotations() {
    // Test function parameter type annotations with generics
    let code = r#"
fn process_option(opt: Option<i32>) -> i32 {
    match opt {
        Some(x) => x,
        None => 0
    }
}
process_option(Some(42))
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok(), "Generic function parameters should work: {:?}", result);
    assert_eq!(result.unwrap(), "42", "Function should extract Some value");
}

#[test]
fn test_generic_type_constructors() {
    // Test that generic type constructors work
    let constructors = [
        "Some(42)",
        "None", 
        "[1, 2, 3]",
        r#"{"a": 1}"#,
    ];
    
    for constructor in &constructors {
        let result = eval_in_repl(constructor);
        assert!(result.is_ok(), "Constructor {} should work: {:?}", constructor, result);
    }
}