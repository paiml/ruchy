/// ERROR-003: TDD tests for REPL-level return statement handling
///
/// Tests that return statements at REPL top-level are handled gracefully
/// instead of producing "Runtime error: return X" messages.
use ruchy::runtime::repl::Repl;
use tempfile::TempDir;

fn create_repl() -> Repl {
    let temp_dir = TempDir::new().unwrap();
    Repl::new(temp_dir.path().to_path_buf()).unwrap()
}

fn eval(repl: &mut Repl, code: &str) -> Result<String, String> {
    repl.eval(code).map_err(|e| e.to_string())
}

#[test]
fn test_function_with_early_return_called() {
    let mut repl = create_repl();
    let code = r#"
        fun safe_divide(a: i32, b: i32) -> i32 {
            if b == 0 {
                return 0;
            }
            a / b
        }
        safe_divide(10, 0)
    "#;
    let result = eval(&mut repl, code).expect("Should execute early return in function");
    assert_eq!(result, "0", "Early return should work in called functions");
}

#[test]
fn test_return_statement_at_top_level() {
    let mut repl = create_repl();
    // Return at top level (not inside function) is an error
    let code = r#"return "hello""#;
    let result = eval(&mut repl, code);

    // Should be a clear error, not "Runtime error: return String(...)"
    assert!(result.is_err(), "Top-level return should be an error");
    let error = result.unwrap_err();
    assert!(
        !error.contains("Runtime error: return"),
        "Error message should not contain 'Runtime error: return', got: {}",
        error
    );
    assert!(
        error.contains("return") || error.contains("top-level") || error.contains("function"),
        "Error should indicate return is not valid at top level, got: {}",
        error
    );
}

#[test]
fn test_function_definition_with_return() {
    let mut repl = create_repl();
    // Defining a function with return should work
    let code = r#"
        fun test() -> String {
            return "hello";
        }
    "#;
    let result = eval(&mut repl, code);
    // Function definition should succeed (returns Unit or function value)
    assert!(
        result.is_ok(),
        "Function definition with return should work: {:?}",
        result
    );
}

#[test]
fn test_function_call_with_return() {
    let mut repl = create_repl();
    // Define function
    eval(
        &mut repl,
        "fun get_message() -> String { return \"success\"; }",
    )
    .unwrap();

    // Call function
    let result = eval(&mut repl, "get_message()").expect("Function call should work");
    assert_eq!(
        result, "\"success\"",
        "Function with return should return correct value"
    );
}

#[test]
fn test_nested_function_with_early_return() {
    let mut repl = create_repl();
    let code = r#"
        fun validate_input(username: String) -> String {
            if username.len() == 0 {
                return "anonymous";
            }
            if username.len() < 3 {
                return "user123";
            }
            username
        }
        validate_input("")
    "#;
    let result = eval(&mut repl, code).expect("Nested early returns should work");
    assert_eq!(result, "\"anonymous\"", "First early return should execute");
}

#[test]
fn test_return_in_loop_inside_function() {
    let mut repl = create_repl();
    let code = r#"
        fun find_first_positive(numbers: Vec<i32>) -> i32 {
            for n in numbers {
                if n > 0 {
                    return n;
                }
            }
            return -1;
        }
        find_first_positive([-1, -2, 3, 4])
    "#;
    let result = eval(&mut repl, code).expect("Return in loop should work");
    assert_eq!(result, "3", "Should return first positive number");
}
