//! Unit tests for function call evaluation in REPL

#[cfg(test)]
#[allow(clippy::panic)] // Tests may panic on failure
#[allow(clippy::expect_used)] // Tests use expect for error messages
mod tests {
    use super::super::*;
    use anyhow::Result;

    /// Test basic println function call
    #[test]
    fn test_println_basic() -> Result<()> {
        let mut repl = Repl::new()?;
        let result = repl.eval(r#"println("Hello, World!")"#)?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Test println with multiple arguments
    #[test]
    fn test_println_multiple_args() -> Result<()> {
        let mut repl = Repl::new()?;
        let result = repl.eval(r#"println("Hello", "World", "!")"#)?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Test println with variables
    #[test]
    fn test_println_with_variables() -> Result<()> {
        let mut repl = Repl::new()?;
        repl.eval("let x = 42")?;
        let result = repl.eval("println(x)")?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Test println with expressions
    #[test]
    fn test_println_with_expressions() -> Result<()> {
        let mut repl = Repl::new()?;
        let result = repl.eval("println(2 + 3)")?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Test println with different value types
    #[test]
    fn test_println_different_types() -> Result<()> {
        let mut repl = Repl::new()?;
        
        // Integer
        let result = repl.eval("println(42)")?;
        assert_eq!(result, "()");
        
        // Float
        let result = repl.eval("println(3.14)")?;
        assert_eq!(result, "()");
        
        // Boolean
        let result = repl.eval("println(true)")?;
        assert_eq!(result, "()");
        
        // String
        let result = repl.eval(r#"println("test")"#)?;
        assert_eq!(result, "()");
        
        Ok(())
    }

    /// Test print function (without newline)
    #[test]
    fn test_print_function() -> Result<()> {
        let mut repl = Repl::new()?;
        let result = repl.eval(r#"print("Hello")"#)?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Test print with multiple arguments
    #[test]
    fn test_print_multiple_args() -> Result<()> {
        let mut repl = Repl::new()?;
        let result = repl.eval(r#"print("A", "B", "C")"#)?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Test unknown function error
    #[test]
    fn test_unknown_function_error() {
        let Ok(mut repl) = Repl::new() else {
            panic!("REPL creation should succeed");
        };
        let result = repl.eval("unknown_function()");
        assert!(result.is_err());
        if let Err(error) = result {
            let error_msg = error.to_string();
            assert!(error_msg.contains("Unknown function"));
        }
    }

    /// Test complex function calls with nested expressions
    #[test]
    fn test_complex_function_calls() -> Result<()> {
        let mut repl = Repl::new()?;
        repl.eval("let x = 10")?;
        repl.eval("let y = 20")?;
        let result = repl.eval("println(x + y, x * y)")?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Test function calls with string interpolation-like behavior
    #[test]
    fn test_function_call_string_formatting() -> Result<()> {
        let mut repl = Repl::new()?;
        repl.eval("let name = \"World\"")?;
        let result = repl.eval(r#"println("Hello,", name, "!")"#)?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Test function calls return unit type
    #[test]
    fn test_function_calls_return_unit() -> Result<()> {
        let mut repl = Repl::new()?;
        let result = repl.eval(r#"println("test")"#)?;
        assert_eq!(result, "()");
        
        // Test that we can assign function call results
        let result = repl.eval(r#"let result = println("assign test")"#)?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Test function calls work in expressions
    #[test]
    fn test_function_calls_in_expressions() -> Result<()> {
        let mut repl = Repl::new()?;
        // Function calls in if expressions
        let result = repl.eval(r#"if true { println("true branch") } else { println("false branch") }"#)?;
        assert_eq!(result, "()");
        Ok(())
    }

    /// Property-based test: all function calls should return unit
    #[test]
    fn test_property_all_builtin_calls_return_unit() -> Result<()> {
        let mut repl = Repl::new()?;
        let builtin_calls = vec![
            r#"println("test")"#,
            r#"print("test")"#,
            r"println(42)",
            r"print(3.14)",
            r"println(true)",
            r"print(false)",
        ];
        
        for call in builtin_calls {
            let result = repl.eval(call)?;
            assert_eq!(result, "()", "Function call {call} should return unit");
        }
        Ok(())
    }

    /// Test memory bounds with function calls
    #[test]
    fn test_function_call_memory_bounds() -> Result<()> {
        let mut repl = Repl::new()?;
        // Test with large string arguments
        let large_string = "x".repeat(1000);
        let call = format!(r#"println("{large_string}")"#);
        let result = repl.eval(&call)?;
        assert_eq!(result, "()");
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::panic)] // Tests may panic on failure
#[allow(clippy::expect_used)] // Tests use expect for error messages
mod property_tests {
    use super::super::*;
    use anyhow::Result;
    
    /// Property-like test: All builtin function calls should return unit type
    #[test]
    fn test_builtin_calls_always_return_unit() -> Result<()> {
        let mut repl = Repl::new()?;
        
        let test_cases = vec![
            "println()",
            "println(42)",
            "println(\"test\")",
            "println(true)",
            "println(1, 2, 3)",
            "print(\"hello\")",
            "print(42, \"world\")",
        ];
        
        for call in test_cases {
            let result = repl.eval(call)?;
            assert_eq!(result, "()", "Function call '{call}' should return unit");
        }
        Ok(())
    }
    
    /// Property-like test: Function calls work with various expression types
    #[test]
    fn test_function_calls_with_different_expressions() -> Result<()> {
        let mut repl = Repl::new()?;
        repl.eval("let x = 10")?;
        repl.eval("let y = 20")?;
        
        let test_cases = vec![
            "println(x + y)",
            "println(x * y, x - y)",
            "println(x > y, x < y)",
            "println(x == 10 && y == 20)",
            "print(\"Result: \", x + y * 2)",
        ];
        
        for call in test_cases {
            let result = repl.eval(call)?;
            assert_eq!(result, "()", "Expression call '{call}' should return unit");
        }
        Ok(())
    }
    
    /// Property-like test: Function calls handle edge cases correctly
    #[test]
    fn test_function_call_edge_cases() -> Result<()> {
        let mut repl = Repl::new()?;
        
        // Empty arguments
        let result = repl.eval("println()")?;
        assert_eq!(result, "()");
        
        // Single arguments of each type
        let result = repl.eval("println(42)")?;
        assert_eq!(result, "()");
        
        let result = repl.eval("println(3.14)")?;
        assert_eq!(result, "()");
        
        let result = repl.eval(r#"println("string")"#)?;
        assert_eq!(result, "()");
        
        let result = repl.eval("println(true)")?;
        assert_eq!(result, "()");
        
        Ok(())
    }
    
    /// Property-like test: print and println have consistent return types
    #[test]
    fn test_print_vs_println_consistency() -> Result<()> {
        let mut repl = Repl::new()?;
        
        let args_list = vec![
            r#""hello""#,
            "42",
            "true",
            r#""a", "b", "c""#,
            "1, 2, 3",
        ];
        
        for args in args_list {
            let print_call = format!("print({args})");
            let println_call = format!("println({args})");
            
            let print_result = repl.eval(&print_call)?;
            let println_result = repl.eval(&println_call)?;
            
            assert_eq!(print_result, "()", "print({args}) should return unit");
            assert_eq!(println_result, "()", "println({args}) should return unit");
            assert_eq!(print_result, println_result, "print and println should have same return type");
        }
        
        Ok(())
    }
}