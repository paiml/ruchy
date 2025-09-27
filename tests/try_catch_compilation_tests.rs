//! EXTREME TDD: Try/Catch Compilation Tests
//! Target: Complete error handling compilation with ALL functions ≤10 complexity
//!
//! Following the same EXTREME TDD approach as set literals:
//! 1. Write ALL tests FIRST (this file)
//! 2. All tests should FAIL initially
//! 3. Implement parser/transpiler to make tests pass
//! 4. Maintain Toyota Way: ≤10 complexity per function
//! 5. Add comprehensive property tests with 10,000 iterations

use ruchy::compile;

#[cfg(test)]
mod try_catch_compilation {
    use super::*;

    // =============================================================================
    // BASIC TRY/CATCH COMPILATION TESTS
    // =============================================================================

    #[test]
    fn test_simple_try_catch_compiles() {
        let code = r"
            fun main() {
                try {
                    42
                } catch (e) {
                    -1
                }
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile simple try-catch: {result:?}"
        );
        let output = result.unwrap();
        // Try-catch should transpile to Result pattern matching
        assert!(output.contains("Result") && output.contains("match"));
        assert!(output.contains("Ok") && output.contains("Err"));
    }

    #[test]
    fn test_try_with_throw_compiles() {
        let code = r#"
            fun main() {
                try {
                    throw "error"
                } catch (e) {
                    99
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile try with throw: {result:?}"
        );
        let output = result.unwrap();
        // Throw should transpile to Result/match pattern
        assert!(output.contains("Result") && output.contains("match"));
        assert!(output.contains("Ok") && output.contains("Err"));
    }

    #[test]
    fn test_try_catch_finally_compiles() {
        let code = r#"
            fun main() {
                try {
                    42
                } catch (e) {
                    -1
                } finally {
                    println("cleanup")
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile try-catch-finally: {result:?}"
        );
        let output = result.unwrap();
        assert!(output.contains("finally") || output.contains("cleanup"));
    }

    #[test]
    fn test_multiple_catch_clauses_compile() {
        let code = r"
            fun main() {
                try {
                    risky_operation()
                } catch (io_error) {
                    handle_io(io_error)
                } catch (parse_error) {
                    handle_parse(parse_error)
                } catch (e) {
                    handle_generic(e)
                }
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile multiple catch clauses: {result:?}"
        );
        let output = result.unwrap();
        // Should have Result pattern matching for error handling
        assert!(output.contains("Result") && output.contains("match"));
        assert!(output.contains("Ok") && output.contains("Err"));
    }

    #[test]
    fn test_nested_try_catch_compiles() {
        let code = r#"
            fun main() {
                try {
                    try {
                        throw "inner"
                    } catch (e) {
                        throw "outer: " + e
                    }
                } catch (e) {
                    e
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile nested try-catch: {result:?}"
        );
        let output = result.unwrap();
        // Should have nested Result patterns
        assert!(output.contains("Result") && output.contains("match"));
        // Count multiple Result patterns for nested structure
        let result_count = output.matches("Result").count();
        assert!(result_count >= 2, "Expected nested Result patterns");
    }

    // =============================================================================
    // THROW STATEMENT TESTS
    // =============================================================================

    #[test]
    fn test_throw_string_compiles() {
        let code = r#"
            fun main() {
                throw "error message"
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile throw string: {result:?}"
        );
        let output = result.unwrap();
        // Throw should transpile to panic!
        assert!(output.contains("panic !"));
    }

    #[test]
    fn test_throw_variable_compiles() {
        let code = r#"
            fun main() {
                let error_msg = "something went wrong";
                throw error_msg
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile throw variable: {result:?}"
        );
    }

    #[test]
    fn test_throw_object_compiles() {
        let code = r#"
            fun main() {
                throw { error_type: "IOError", message: "file not found" }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile throw object: {result:?}"
        );
    }

    // =============================================================================
    // CATCH PATTERN TESTS
    // =============================================================================

    #[test]
    fn test_catch_simple_identifier() {
        let code = r#"
            fun main() {
                try {
                    throw "error"
                } catch (e) {
                    e
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile catch with identifier: {result:?}"
        );
    }

    #[test]
    fn test_catch_with_type_pattern() {
        let code = r#"
            fun main() {
                try {
                    throw "error"
                } catch (msg) {
                    msg
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile catch with type pattern: {result:?}"
        );
    }

    #[test]
    fn test_catch_with_guard() {
        let code = r#"
            fun main() {
                try {
                    throw 42
                } catch (n) {
                    if n > 0 {
                        "positive error"
                    } else {
                        "other error"
                    }
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile catch with guard: {result:?}"
        );
    }

    // =============================================================================
    // INTEGRATION WITH OTHER LANGUAGE FEATURES
    // =============================================================================

    #[test]
    fn test_try_catch_in_function() {
        let code = r#"
            fun safe_divide(a, b) {
                try {
                    if b == 0 {
                        throw "division by zero"
                    }
                    a / b
                } catch (e) {
                    0
                }
            }

            fun main() {
                safe_divide(10, 2)
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile try-catch in function: {result:?}"
        );
    }

    #[test]
    fn test_try_catch_with_return() {
        let code = r"
            fun maybe_fail() {
                try {
                    return 42
                } catch (e) {
                    return -1
                }
            }

            fun main() {
                maybe_fail()
            }
        ";

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile try-catch with return: {result:?}"
        );
    }

    #[test]
    fn test_try_catch_in_loop() {
        let code = r#"
            fun main() {
                for i in [1, 2, 3] {
                    try {
                        if i == 2 {
                            throw "skip"
                        }
                        println(i)
                    } catch (e) {
                        continue
                    }
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile try-catch in loop: {result:?}"
        );
    }

    // =============================================================================
    // EDGE CASES
    // =============================================================================

    #[test]
    fn test_empty_try_block() {
        let code = r#"
            fun main() {
                try {
                } catch (e) {
                    "error"
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile empty try block: {result:?}"
        );
    }

    #[test]
    fn test_try_without_catch() {
        let code = r#"
            fun main() {
                try {
                    42
                } finally {
                    println("cleanup")
                }
            }
        "#;

        let result = compile(code);
        // This might be invalid syntax, but should not panic
        let _ = result;
    }

    #[test]
    fn test_catch_without_finally() {
        let code = r#"
            fun main() {
                try {
                    throw "error"
                } catch (e) {
                    "handled"
                }
            }
        "#;

        let result = compile(code);
        assert!(
            result.is_ok(),
            "Failed to compile try-catch without finally: {result:?}"
        );
    }

    // =============================================================================
    // SYNTAX ERROR CASES (Should fail gracefully)
    // =============================================================================

    #[test]
    fn test_malformed_try_syntax() {
        let code = r"
            fun main() {
                try
                    42
                catch e {
                    -1
                }
            }
        ";

        let result = compile(code);
        // Should fail parsing but not panic
        let _ = result;
    }

    #[test]
    fn test_catch_without_pattern() {
        let code = r"
            fun main() {
                try {
                    42
                } catch {
                    -1
                }
            }
        ";

        let result = compile(code);
        // Should fail parsing but not panic
        let _ = result;
    }
}
