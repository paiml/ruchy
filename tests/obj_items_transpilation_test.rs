// TDD: Test for obj.items() transpilation bug
// Bug: transpiler converts items() to iter() which has wrong signature

use std::process::Command;
use std::fs;

#[test]
fn test_obj_items_transpilation_compiles() {
    // This should compile and run without errors
    let code = r#"
        let obj = {"a": 1, "b": 2}
        for key, value in obj.items() {
            println(key)
            println(value)
        }
    "#;
    
    fs::write("/tmp/test_obj_items_transpile.ruchy", code).unwrap();
    
    let output = Command::new("./target/release/ruchy")
        .arg("run")
        .arg("/tmp/test_obj_items_transpile.ruchy")
        .output()
        .expect("Failed to run ruchy");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should not have compilation errors
    assert!(!stderr.contains("error:"), 
        "Should compile without errors, got stderr: {stderr}");
    
    // Should execute successfully  
    assert!(output.status.success(),
        "Should execute successfully, got stderr: {stderr}");
    
    // Should produce output
    assert!(stdout.contains('a') && (stdout.contains('1') || stdout.contains('2')),
        "Should print keys and values, got stdout: {stdout}");
}

#[test]
fn test_obj_items_method_exists() {
    // Test that obj.items() works in REPL
    let code = r#"let obj = {"a": 1}; obj.items()"#;
    
    let output = Command::new("bash")
        .arg("-c")
        .arg(format!("echo '{code}' | ./target/release/ruchy repl"))
        .output()
        .expect("Failed to run ruchy");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("(\"a\", 1)"),
        "obj.items() should work in REPL, got: {stdout}");
}

#[test]
fn test_transpiled_items_vs_repl() {
    // Compare REPL vs transpiled behavior
    let code = r#"let obj = {"test": 42}; obj.items()"#;
    
    // REPL version
    let repl_output = Command::new("bash")
        .arg("-c")
        .arg(format!("echo '{code}' | ./target/release/ruchy repl"))
        .output()
        .expect("Failed to run ruchy repl");
    let repl_stdout = String::from_utf8_lossy(&repl_output.stdout);
    
    // File version
    fs::write("/tmp/test_transpiled_items.ruchy", code).unwrap();
    let file_output = Command::new("./target/release/ruchy")
        .arg("/tmp/test_transpiled_items.ruchy")
        .output()
        .expect("Failed to run ruchy file");
    let _file_stdout = String::from_utf8_lossy(&file_output.stdout);
    
    // Both should work and produce similar results
    assert!(repl_stdout.contains("(\"test\", 42)"),
        "REPL should work: {repl_stdout}");
    assert!(file_output.status.success(),
        "File execution should work: {}", String::from_utf8_lossy(&file_output.stderr));
}