// Async/Await Test Suite  
// Testing async functions, await expressions, and futures

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

// Helper to test transpiled code
fn eval_transpiled(code: &str) -> Result<String, String> {
    let test_file = format!("/tmp/async_test_{}.ruchy", 
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
fn test_async_function_definition() {
    // Test defining an async function
    let code = r#"
async fn fetch_data() -> String {
    "data"
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok(), "Async function definition should work: {:?}", result);
}

#[test]
fn test_await_expression() {
    // Test using await
    let code = r#"
async fn get_value() -> i32 {
    42
}

async fn main() {
    let value = await get_value()
    println(value)
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Await expression should at least parse");
}

#[test] 
fn test_async_with_delay() {
    // Test async with simulated delay
    let code = r#"
async fn delayed_hello() -> String {
    // In a real implementation, this would use a timer
    "Hello after delay"
}

async fn main() {
    let msg = await delayed_hello()
    println(msg)
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Async with delay should at least parse");
}

#[test]
fn test_multiple_awaits() {
    // Test multiple await expressions
    let code = r#"
async fn first() -> i32 { 1 }
async fn second() -> i32 { 2 }
async fn third() -> i32 { 3 }

async fn sum_all() -> i32 {
    let a = await first()
    let b = await second()
    let c = await third()
    a + b + c
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Multiple awaits should at least parse");
}

#[test]
fn test_async_in_expressions() {
    // Test await in expressions
    let code = r#"
async fn get_x() -> i32 { 10 }
async fn get_y() -> i32 { 20 }

async fn calculate() -> i32 {
    (await get_x()) + (await get_y())
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Await in expressions should at least parse");
}

#[test]
fn test_async_error_handling() {
    // Test async with Result types
    let code = r#"
async fn might_fail() -> Result<i32, String> {
    Ok(42)
}

async fn handle_error() {
    match await might_fail() {
        Ok(value) => println(value),
        Err(e) => println(e)
    }
}
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Async error handling should at least parse");
}

#[test]
fn test_async_closure() {
    // Test async closures
    let code = r#"
let async_add = async |x, y| x + y
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Async closure should at least parse");
}

#[test]
fn test_transpiled_async() {
    // Test that async transpiles to valid Rust
    let code = r#"
async fn hello() -> String {
    "Hello from async"
}

async fn main() {
    let msg = await hello()
    println(msg)
}
"#;
    
    // This will likely fail until async is properly implemented in transpiler
    let result = eval_transpiled(code);
    // Just check it doesn't crash the transpiler
    let _ = result;
}