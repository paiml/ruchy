/// Lint tool demonstration
/// Shows various code quality issues that the lint tool can detect

use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn main() {
    println!("=== Ruchy Lint Tool Demo ===\n");
    
    demo_unused_variables();
    demo_undefined_variables();
    demo_variable_shadowing();
    demo_unused_parameters();
    demo_unused_loop_variables();
    demo_unused_match_bindings();
    demo_clean_code();
}

fn demo_unused_variables() {
    println!("1. Detecting Unused Variables:");
    let code = r#"
fn test() {
    let unused = 42;  // This is never used
    let used = 1;
    println(used);
}
"#;
    run_lint(code, "unused_vars.ruchy");
}

fn demo_undefined_variables() {
    println!("\n2. Detecting Undefined Variables:");
    let code = r#"
fn test() {
    println(undefined_var);  // This was never defined
}
"#;
    run_lint(code, "undefined_vars.ruchy");
}

fn demo_variable_shadowing() {
    println!("\n3. Detecting Variable Shadowing:");
    let code = r#"
fn test() {
    let x = 1;
    let x = 2;  // This shadows the previous x
    println(x);
}
"#;
    run_lint(code, "shadowing.ruchy");
}

fn demo_unused_parameters() {
    println!("\n4. Detecting Unused Parameters:");
    let code = r#"
fn add(a, b, unused) {
    a + b  // 'unused' parameter is never used
}
"#;
    run_lint(code, "unused_params.ruchy");
}

fn demo_unused_loop_variables() {
    println!("\n5. Detecting Unused Loop Variables:");
    let code = r#"
fn test() {
    for i in 0..10 {
        // i is never used in the loop body
    }
}
"#;
    run_lint(code, "unused_loop.ruchy");
}

fn demo_unused_match_bindings() {
    println!("\n6. Detecting Unused Match Bindings:");
    let code = r#"
fn test(x) {
    match x {
        1 => {},
        y => {}  // y is bound but never used
    }
}
"#;
    run_lint(code, "unused_match.ruchy");
}

fn demo_clean_code() {
    println!("\n7. Clean Code (No Issues):");
    let code = r#"
fn add(a, b) {
    a + b
}
"#;
    run_lint(code, "clean.ruchy");
}

fn run_lint(code: &str, filename: &str) {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join(filename);
    
    // Write the code to a file
    fs::write(&file_path, code).unwrap();
    
    // Run the lint command
    let output = Command::new("./target/release/ruchy")
        .args(&["lint", file_path.to_str().unwrap()])
        .output()
        .expect("Failed to run ruchy lint");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    println!("Code: {}", code.trim());
    println!("Result:");
    
    if !stdout.is_empty() {
        println!("{}", stdout);
    }
    if !stderr.is_empty() {
        eprintln!("Error: {}", stderr);
    }
}