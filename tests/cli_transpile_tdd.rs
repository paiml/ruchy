// TDD test for CLI transpile command issues
// Focuses on the actual CLI code path that users experience

use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_cli_transpile_match_expression() {
    // RED: Create failing test for CLI transpile with match expression
    let input = r#"let number = 2
match number {
    1 => println("One"),
    2 => println("Two"),
    _ => println("Other")
}
"#;
    
    // Write input to temporary file
    let mut input_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(input_file.path(), input).expect("Failed to write input");
    
    // Run CLI transpile command
    let output = Command::new("cargo")
        .args(&["run", "--", "transpile"])
        .arg(input_file.path())
        .output()
        .expect("Failed to execute CLI command");
    
    let rust_code = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    
    // Should not contain double semicolons
    assert!(!rust_code.contains(" ; ;"), 
            "CLI should not generate double semicolons: {}", rust_code);
    
    // Should handle Unit types properly
    assert!(rust_code.contains(r#""()" => {}"#) || !rust_code.contains(r#"println!("{}", result)"#),
            "CLI should handle Unit types without Display error: {}", rust_code);
    
    // Generated code should compile
    let mut temp_rust = NamedTempFile::with_suffix(".rs").expect("Failed to create temp rust file");
    fs::write(temp_rust.path(), &rust_code).expect("Failed to write rust code");
    
    let compile_result = Command::new("rustc")
        .arg(temp_rust.path())
        .arg("--crate-name")
        .arg("test_crate")
        .output()
        .expect("Failed to run rustc");
    
    if !compile_result.status.success() {
        let error = String::from_utf8_lossy(&compile_result.stderr);
        panic!("Generated code should compile successfully. Compilation error:\n{}\n\nGenerated code:\n{}", error, rust_code);
    }
}

#[test]
fn test_cli_transpile_simple_let() {
    // Test simple let statement via CLI
    let input = r#"let x = 5
x + 1
"#;
    
    let mut input_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(input_file.path(), input).expect("Failed to write input");
    
    let output = Command::new("cargo")
        .args(&["run", "--", "transpile"])
        .arg(input_file.path())
        .output()
        .expect("Failed to execute CLI command");
    
    let rust_code = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    
    // Should not contain double semicolons
    assert!(!rust_code.contains(" ; ;"), 
            "CLI should not generate double semicolons for simple let: {}", rust_code);
    
    // Should compile successfully
    let mut temp_rust = NamedTempFile::with_suffix(".rs").expect("Failed to create temp rust file");
    fs::write(temp_rust.path(), &rust_code).expect("Failed to write rust code");
    
    let compile_result = Command::new("rustc")
        .arg(temp_rust.path())
        .arg("--crate-name")
        .arg("test_crate")
        .output()
        .expect("Failed to run rustc");
    
    assert!(compile_result.status.success(), 
            "Generated code should compile. Error: {}", 
            String::from_utf8_lossy(&compile_result.stderr));
}

#[test]
fn test_cli_transpile_println_statements() {
    // Test println statements returning Unit type
    let input = r#"println("Hello")
println("World")
"#;
    
    let mut input_file = NamedTempFile::new().expect("Failed to create temp file");
    fs::write(input_file.path(), input).expect("Failed to write input");
    
    let output = Command::new("cargo")
        .args(&["run", "--", "transpile"])
        .arg(input_file.path())
        .output()
        .expect("Failed to execute CLI command");
    
    let rust_code = String::from_utf8(output.stdout).expect("Invalid UTF-8");
    
    // Should handle Unit types from println without Display formatting error
    assert!(!rust_code.contains(r#"println!("{}", result)"#) || rust_code.contains(r#""()" => {}"#),
            "CLI should handle Unit types from println: {}", rust_code);
    
    // Should compile successfully  
    let mut temp_rust = NamedTempFile::with_suffix(".rs").expect("Failed to create temp rust file");
    fs::write(temp_rust.path(), &rust_code).expect("Failed to write rust code");
    
    let compile_result = Command::new("rustc")
        .arg(temp_rust.path())
        .arg("--crate-name")
        .arg("test_crate")
        .output()
        .expect("Failed to run rustc");
    
    assert!(compile_result.status.success(), 
            "Generated code should compile. Error: {}", 
            String::from_utf8_lossy(&compile_result.stderr));
}