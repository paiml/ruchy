// TDD RED: Multi-arg println bug test - should fail initially
// Tests RUCHY-100: Multi-arg println transpiler bug

use ruchy::backend::transpiler::Transpiler;
use ruchy::frontend::parser::Parser;
use std::process::Command;
use std::fs;

#[test]
fn test_multi_arg_println_transpilation() {
    let code = r#"println("Hello", "World", "from", "Ruchy")"#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast).expect("Should transpile");
    let rust_code = result.to_string();
    
    println!("Generated Rust code: {}", rust_code);
    
    // The bug: current code generates println!("Hello", "World", "from", "Ruchy") 
    // which treats "Hello" as format string, causing compilation errors
    assert!(!rust_code.contains(r#"println ! ( "Hello" , "World" , "from" , "Ruchy" )"#), 
            "Should not generate invalid Rust code where first arg becomes format string");
}

#[test] 
fn test_multi_arg_println_execution() {
    let code = r#"println("Hello", "World", "from", "Ruchy")"#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast).expect("Should transpile");
    let rust_code = result.to_string();
    
    // Write to temp file and try to compile
    let temp_file = "/tmp/test_multi_println.rs";
    fs::write(temp_file, rust_code).expect("Should write temp file");
    
    let output = Command::new("rustc")
        .arg(temp_file)
        .arg("-o")
        .arg("/tmp/test_multi_println")
        .output()
        .expect("Should run rustc");
    
    // The bug: this should compile successfully but currently fails
    assert!(output.status.success(), 
            "Generated Rust code should compile successfully. Stderr: {}", 
            String::from_utf8_lossy(&output.stderr));
}

#[test]
fn test_multi_arg_println_output() {
    let code = r#"println("Hello", "World", "from", "Ruchy")"#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast).expect("Should transpile");
    let rust_code = result.to_string();
    
    // Write and compile
    let temp_rs = "/tmp/test_multi_println_output.rs";
    let temp_exe = "/tmp/test_multi_println_output";
    fs::write(temp_rs, rust_code).expect("Should write temp file");
    
    let compile_output = Command::new("rustc")
        .arg(temp_rs)
        .arg("-o")
        .arg(temp_exe)
        .output()
        .expect("Should run rustc");
    
    if compile_output.status.success() {
        // Run and check output
        let run_output = Command::new(temp_exe)
            .output()
            .expect("Should run executable");
            
        let stdout = String::from_utf8_lossy(&run_output.stdout);
        
        // Expected: "Hello World from Ruchy" (space-separated)
        // Current bug: "Hello\n" (only first arg, others ignored/dumped)
        assert!(stdout.contains("Hello World from Ruchy") || stdout.contains("Hello\nWorld\nfrom\nRuchy"),
                "Should print all arguments. Got: '{}'", stdout);
    } else {
        panic!("Code should compile. Rustc error: {}", String::from_utf8_lossy(&compile_output.stderr));
    }
}

#[test]
fn test_two_arg_println() {
    // Simpler case to isolate the issue
    let code = r#"println("Hello", "World")"#;
    
    let mut parser = Parser::new(code);
    let ast = parser.parse().expect("Should parse");
    
    let transpiler = Transpiler::new();
    let result = transpiler.transpile_to_program(&ast).expect("Should transpile");
    let rust_code = result.to_string();
    
    // Write and compile
    let temp_rs = "/tmp/test_two_println.rs";
    let temp_exe = "/tmp/test_two_println";
    fs::write(temp_rs, rust_code).expect("Should write temp file");
    
    let compile_output = Command::new("rustc")
        .arg(temp_rs) 
        .arg("-o")
        .arg(temp_exe)
        .output()
        .expect("Should run rustc");
    
    assert!(compile_output.status.success(),
            "Two-arg println should compile. Error: {}", 
            String::from_utf8_lossy(&compile_output.stderr));
            
    if compile_output.status.success() {
        let run_output = Command::new(temp_exe)
            .output()
            .expect("Should run executable");
            
        let stdout = String::from_utf8_lossy(&run_output.stdout);
        assert!(stdout.contains("Hello World") || stdout.contains("Hello\nWorld"),
                "Should print both arguments. Got: '{}'", stdout);
    }
}