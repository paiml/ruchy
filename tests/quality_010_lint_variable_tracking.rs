/// TDD: Variable tracking tests for lint tool
/// These tests define the EXACT behavior we expect from lint variable tracking
use std::fs;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn test_lint_detects_unused_variables() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Unused variable should be detected
    let code = r"
fn test() {
    let unused = 42;  // Never used
    let used = 1;
    println(used);
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unused variable: unused"), "Should detect unused variable");
    assert!(!stdout.contains("unused variable: used"), "Should not flag used variable");
}

#[test]
fn test_lint_detects_undefined_variables() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Using undefined variable
    let code = r"
fn test() {
    println(undefined_var);  // Never defined
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("undefined variable: undefined_var"), "Should detect undefined variable");
}

#[test]
fn test_lint_tracks_variable_scope() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Variable defined in inner scope, used in outer
    let code = r"
fn test() {
    if true {
        let inner = 42;
    }
    println(inner);  // Should be undefined here
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("undefined variable: inner"), "Should detect out-of-scope variable");
}

#[test]
fn test_lint_tracks_shadowed_variables() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Variable shadowing
    let code = r"
fn test() {
    let x = 1;
    let x = 2;  // Shadows previous x
    println(x);
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("variable shadowing: x"), "Should warn about variable shadowing");
}

#[test]
fn test_lint_tracks_function_parameters() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Unused function parameter
    let code = r"
fn add(a: i32, b: i32, unused: i32) -> i32 {
    a + b
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unused parameter: unused"), "Should detect unused parameter");
}

#[test]
fn test_lint_tracks_loop_variables() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Loop variable usage
    let code = r"
fn test() {
    for i in 0..10 {
        // i is never used in loop body
    }
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("unused loop variable: i"), "Should detect unused loop variable");
}

#[test]
fn test_lint_tracks_match_bindings() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Match binding usage (simplified syntax for parser)
    let code = r"
fn test(opt) {
    match opt {
        Some(value) => {},  // value not used
        None => {}
    }
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("Match test stdout: {stdout}");
    eprintln!("Match test stderr: {stderr}");
    assert!(stdout.contains("unused match binding: value"), "Should detect unused match binding");
}

#[test]
fn test_lint_json_output() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    let code = r"
fn test() {
    let unused = 42;
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    
    assert!(json["issues"].is_array(), "Should have issues array");
    let issues = json["issues"].as_array().unwrap();
    assert!(!issues.is_empty(), "Should have at least one issue");
    
    let first_issue = &issues[0];
    assert_eq!(first_issue["type"], "unused_variable");
    assert_eq!(first_issue["name"], "unused");
    assert!(first_issue["line"].is_number());
}

#[test]
fn test_lint_clean_code() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    // Clean code with no issues (simplified syntax)
    let code = r"
fn add(a, b) {
    a + b
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No issues found") || stdout.contains("✓"), 
            "Clean code should report no issues");
}

#[test]
fn test_lint_severity_levels() {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("test.ruchy");
    
    let code = r"
fn test() {
    let unused = 42;        // Warning
    println(undefined);     // Error
}
";
    
    fs::write(&file_path, code).unwrap();
    
    let output = Command::new("./target/debug/ruchy")
        .args(["lint", file_path.to_str().unwrap()])
        .output()
        .unwrap();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Warning") || stdout.contains("warning"), "Should show warnings");
    assert!(stdout.contains("Error") || stdout.contains("error"), "Should show errors");
}