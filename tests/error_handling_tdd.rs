//! TDD tests for error handling features (try-catch, throw, panic, Result)
//! Target: Ch17 compatibility

use ruchy::runtime::repl::Repl;
use std::env;

// ============================================================================
// Try-Catch Tests
// ============================================================================

#[test]
fn test_try_catch_basic() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        try {
            42
        } catch (e) {
            0
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_try_catch_with_error() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        try {
            let x = undefined_variable
            x
        } catch (e) {
            "Error caught"
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "\"Error caught\"");
}

#[test]
fn test_try_catch_finally() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        let mut x = 0
        try {
            x = 1
            42
        } catch (e) {
            x = 2
            0
        } finally {
            x = 3
        }
        x
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "3");
}

#[test]
fn test_nested_try_catch() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        try {
            try {
                undefined_var
            } catch (inner) {
                "Inner caught"
            }
        } catch (outer) {
            "Outer caught"
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "\"Inner caught\"");
}

// ============================================================================
// Throw Tests
// ============================================================================

#[test]
fn test_throw_string() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        try {
            throw "Custom error"
            42
        } catch (e) {
            "Caught throw"
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "\"Caught throw\"");
}

#[test]
fn test_throw_caught() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        try {
            throw "Custom error"
        } catch (e) {
            "Caught: error"
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "\"Caught: error\"");
}

// ============================================================================
// Panic Tests
// ============================================================================

#[test]
fn test_panic_macro() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        try {
            panic!("Panic message")
            42
        } catch (e) {
            "Panic caught"
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "\"Panic caught\"");
}

// ============================================================================
// Result Type Tests
// ============================================================================

#[test]
fn test_result_ok() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        let result = Ok(42)
        match result {
            Ok(val) => val,
            Err(_) => 0
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_result_err() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        let result = Err("Something went wrong")
        match result {
            Ok(val) => val,
            Err(msg) => msg
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "\"Something went wrong\"");
}

#[test]
fn test_result_unwrap() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        let result = Ok(42)
        result.unwrap()
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_result_unwrap_or() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        let result = Err("Error")
        result.unwrap_or(0)
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "0");
}

#[test]
fn test_result_is_ok() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        let result = Ok(42)
        result.is_ok()
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "true");
}

#[test]
fn test_result_is_err() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        let result = Err("Error")
        result.is_err()
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "true");
}

// ============================================================================
// Question Mark Operator Tests
// ============================================================================

#[test]
fn test_question_mark_operator() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        fun divide(a, b) {
            if b == 0 {
                Err("Division by zero")
            } else {
                Ok(a / b)
            }
        }
        
        fun calculate() {
            let x = divide(10, 2)?
            let y = divide(x, 2)?
            Ok(y)
        }
        
        match calculate() {
            Ok(val) => val,
            Err(e) => 0
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "2.5");
}

#[test]
fn test_question_mark_early_return() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        fun divide(a, b) {
            if b == 0 {
                Err("Division by zero")
            } else {
                Ok(a / b)
            }
        }
        
        fun calculate() {
            let x = divide(10, 0)?  // Should return early
            let y = divide(x, 2)?
            Ok(y)
        }
        
        match calculate() {
            Ok(val) => val,
            Err(e) => -1
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "-1");
}

// ============================================================================
// Error Propagation Tests
// ============================================================================

#[test]
fn test_error_propagation() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        fun level3() {
            throw "Level 3 error"
        }
        
        fun level2() {
            level3()
        }
        
        fun level1() {
            level2()
        }
        
        try {
            level1()
        } catch (e) {
            "Caught from level 3"
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "\"Caught from level 3\"");
}

#[test]
fn test_mixed_error_handling() {
    let mut repl = Repl::new(std::env::temp_dir()).unwrap();
    let code = r#"
        fun risky_operation(should_fail) {
            if should_fail {
                Err("Operation failed")
            } else {
                Ok("Success")
            }
        }
        
        try {
            let result1 = risky_operation(false)?
            let result2 = risky_operation(true)?
            Ok(result1 + " and " + result2)
        } catch (e) {
            "Caught error"
        }
    "#;
    let result = repl.eval(code).unwrap();
    assert_eq!(result, "\"Caught error\"");
}