// Standard Library Validation Test Suite
// This validates that stdlib methods work in both interpreter AND transpiler

use std::process::Command;
use std::fs;

fn test_in_repl(code: &str) -> Result<String, String> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("echo '{code}' | ./target/release/ruchy repl 2>&1"))
        .output()
        .map_err(|e| format!("Failed to run REPL: {e}"))?;
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Extract the result (skip welcome message and prompts)
    let lines: Vec<&str> = stdout.lines()
        .filter(|line| {
            !line.contains("Welcome to Ruchy") &&
            !line.contains("Type :help") &&
            !line.contains("Goodbye") &&
            !line.is_empty() &&
            !line.trim().is_empty()
        })
        .collect();
    
    if lines.is_empty() {
        return Err(format!("No output from REPL. Full output: {stdout}"));
    }
    
    // Return the last non-empty line (the result)
    Ok(lines[lines.len() - 1].to_string())
}

fn test_in_file(code: &str) -> Result<String, String> {
    let test_file = "/tmp/stdlib_validation_test.ruchy";
    fs::write(test_file, code)
        .map_err(|e| format!("Failed to write test file: {e}"))?;
    
    let output = Command::new("./target/release/ruchy")
        .arg(test_file)
        .output()
        .map_err(|e| format!("Failed to run file: {e}"))?;
    
    if !output.status.success() {
        return Err(format!("File execution failed: {}", 
            String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn validate_method(name: &str, code: &str, expected: &str) -> Result<(), String> {
    // Test in REPL
    let repl_result = test_in_repl(code)?;
    if !repl_result.contains(expected) {
        return Err(format!("{name}: REPL returned '{repl_result}', expected '{expected}'"));
    }
    
    // Test in file (transpiled)
    match test_in_file(code) {
        Ok(file_result) => {
            if file_result != expected {
                return Err(format!("{name}: File execution returned '{file_result}', expected '{expected}'"));
            }
            Ok(())
        }
        Err(e) => {
            // If file execution fails, it means transpiler doesn't support it
            Err(format!("{name}: Works in REPL but NOT in transpiler: {e}"))
        }
    }
}

#[test]
fn validate_string_methods() {
    let tests = vec![
        ("string_len", r#"println("hello".len())"#, "5"),
        ("string_to_upper", r#"println("hello".to_upper())"#, "HELLO"),
        ("string_to_lower", r#"println("HELLO".to_lower())"#, "hello"),
        ("string_trim", r#"println("  hello  ".trim())"#, "hello"),
    ];
    
    let mut failures = Vec::new();
    
    for (name, code, expected) in tests {
        if let Err(e) = validate_method(name, code, expected) {
            failures.push(e);
        }
    }
    
    assert!(failures.is_empty(), "Standard library validation failures:\n{}", failures.join("\n"));
}

#[test]
fn validate_array_methods() {
    let tests = vec![
        ("array_len", r"println([1,2,3].len())", "3"),
        ("array_first", r"println([1,2,3].first())", "1"),
        ("array_last", r"println([1,2,3].last())", "3"),
    ];
    
    let mut failures = Vec::new();
    
    for (name, code, expected) in tests {
        if let Err(e) = validate_method(name, code, expected) {
            failures.push(e);
        }
    }
    
    assert!(failures.is_empty(), "Array method validation failures:\n{}", failures.join("\n"));
}

#[test]
fn validate_number_methods() {
    let tests = vec![
        ("int_to_string", r"println(42.to_string())", "42"),
        ("int_abs", r"println((-5).abs())", "5"),
        ("float_floor", r"println(3.7.floor())", "3"),
        ("float_ceil", r"println(3.2.ceil())", "4"),
    ];
    
    let mut failures = Vec::new();
    
    for (name, code, expected) in tests {
        if let Err(e) = validate_method(name, code, expected) {
            failures.push(e);
        }
    }
    
    assert!(failures.is_empty(), "Number method validation failures:\n{}", failures.join("\n"));
}

#[test]
fn validate_object_methods() {
    // These should work in both REPL and transpiler after v1.20.1
    let tests = vec![
        ("object_keys", r#"let obj = {"a": 1}; for k in obj.keys() { println(k) }"#, "a"),
        ("object_values", r#"let obj = {"a": 42}; for v in obj.values() { println(v) }"#, "42"),
        ("object_items", r#"let obj = {"key": 1}; for k, v in obj.items() { println(k) }"#, "key"),
    ];
    
    let mut failures = Vec::new();
    
    for (name, code, expected) in tests {
        if let Err(e) = validate_method(name, code, expected) {
            failures.push(e);
        }
    }
    
    assert!(failures.is_empty(), "Object method validation failures:\n{}", failures.join("\n"));
}