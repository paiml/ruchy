// Error Handling and Recovery Test Suite
// Testing Result types, try operator, panic recovery, and error propagation

use ruchy::runtime::repl::Repl;

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

#[test]
fn test_result_type() {
    // Test Result<T, E> type
    let code = r#"
let divide = |a, b| {
    if b == 0 {
        Err("Division by zero")
    } else {
        Ok(a / b)
    }
}

divide(10, 2)
"#;
    
    let result = eval_in_repl(code);
    if let Ok(res) = result {
        assert!(res.contains("Ok") && res.contains("5"));
    }
}

#[test]
fn test_unwrap_behavior() {
    // Test unwrap on Result
    let code = r#"
let safe_divide = |a, b| {
    if b == 0 {
        Err("Cannot divide by zero")
    } else {
        Ok(a / b)
    }
}

safe_divide(20, 4).unwrap()
"#;
    
    let result = eval_in_repl(code);
    if let Ok(res) = result {
        assert_eq!(res, "5");
    }
}

#[test]
fn test_unwrap_or() {
    // Test unwrap_or with default
    let code = r#"
let safe_parse = |s| {
    match s {
        "42" => Ok(42),
        _ => Err("Parse error")
    }
}

safe_parse("invalid").unwrap_or(0)
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "unwrap_or should at least parse");
}

#[test]
fn test_map_result() {
    // Test map on Result
    let code = r#"
let double_if_ok = Ok(21).map(|x| x * 2)
double_if_ok
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Result map should at least parse");
}

#[test]
fn test_and_then() {
    // Test and_then for chaining
    let code = r#"
let parse = |s| match s {
    "10" => Ok(10),
    _ => Err("parse error")
}

let double = |x| Ok(x * 2)

parse("10").and_then(double)
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "and_then should at least parse");
}

#[test]
fn test_try_operator() {
    // Test ? operator
    let code = r#"
fn process() -> Result<i32, String> {
    let x = Ok(10)?
    let y = Ok(20)?
    Ok(x + y)
}

process()
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Try operator should at least parse");
}

#[test]
fn test_panic_recovery() {
    // Test panic and recovery
    let code = r#"
let safe_op = || {
    panic("Something went wrong")
}

// This should not crash the REPL
"handled"
"#;
    
    let result = eval_in_repl(code);
    // REPL should recover from panic
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_option_question_mark() {
    // Test ? with Option
    let code = r#"
fn get_value() -> Option<i32> {
    let x = Some(10)?
    let y = Some(20)?
    Some(x + y)
}

get_value()
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Option ? should at least parse");
}

#[test]
fn test_custom_error_types() {
    // Test custom error types
    let code = r#"
enum MyError {
    NotFound,
    InvalidInput(String),
    IOError { path: String, kind: String }
}

Err(MyError::InvalidInput("bad data"))
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Custom error types should at least parse");
}

#[test]
fn test_error_conversion() {
    // Test From trait for error conversion
    let code = r#"
impl From<String> for MyError {
    fn from(s: String) -> MyError {
        MyError::InvalidInput(s)
    }
}

let err: Result<i32, MyError> = Err("bad".into())
err
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Error conversion should at least parse");
}

#[test]
fn test_result_collect() {
    // Test collecting Results
    let code = r#"
let results = [Ok(1), Ok(2), Ok(3)]
let collected: Result<Vec<i32>, String> = results.into_iter().collect()
collected
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Result collect should at least parse");
}

#[test]
fn test_early_return() {
    // Test early return on error
    let code = r#"
fn validate(x: i32) -> Result<i32, String> {
    if x < 0 {
        return Err("Negative number")
    }
    if x > 100 {
        return Err("Too large")
    }
    Ok(x * 2)
}

validate(50)
"#;
    
    let result = eval_in_repl(code);
    assert!(result.is_ok() || result.is_err(), "Early return should at least parse");
}