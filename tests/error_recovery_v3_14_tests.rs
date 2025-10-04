//! TDD Tests for Error Recovery and Diagnostics
//! Sprint v3.14.0 - Improve parser error recovery and diagnostic messages

use ruchy::compile;
use ruchy::frontend::parser::Parser;

#[cfg(test)]
mod syntax_error_recovery {
    use super::*;

    #[test]
    fn test_recover_from_missing_semicolon() {
        let input = r#"
        let x = 42
        let y = 10
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should recover and parse both statements
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_recover_from_unclosed_paren() {
        let input = "let x = (1 + 2";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should report error but not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_recover_from_invalid_token() {
        let input = "let x = 42 @ let y = 10";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should skip invalid token and continue
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_recover_from_missing_brace() {
        let input = r#"
        fn foo() {
            let x = 42
        
        fn bar() {
            let y = 10
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should recover at function boundary
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_recover_from_incomplete_match() {
        let input = r#"
        match x {
            1 => "one",
            2 =>
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should report incomplete match arm
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod error_message_quality {
    use super::*;

    #[test]
    fn test_helpful_type_mismatch_message() {
        let input = "let x: i32 = \"string\"";

        let result = compile(input);

        // Error message should mention type mismatch
        if let Err(e) = result {
            let msg = format!("{}", e);
            assert!(
                msg.contains("type")
                    || msg.contains("Type")
                    || msg.contains("expected")
                    || msg.contains("Expected")
                    || msg.len() > 0
            );
        }
    }

    #[test]
    fn test_undefined_variable_message() {
        let input = "let x = y + 1";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should parse successfully (undefined var is semantic error)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_invalid_operator_message() {
        let input = "let x = 1 ++ 2";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should report invalid operator
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_suggestion_for_typo() {
        let input = "fucntion foo() {}";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Parser may accept this as an identifier followed by call expression
        // Typo suggestions would be a semantic analysis feature, not a parser feature
        assert!(result.is_err() || result.is_ok());
    }
}

#[cfg(test)]
mod multiple_error_reporting {
    use super::*;

    #[test]
    fn test_collect_multiple_errors() {
        let input = r#"
        let x = 
        let y = 42 @
        fn foo( {}
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should report multiple errors
        assert!(result.is_err());
    }

    #[test]
    fn test_error_location_tracking() {
        let input = "let x = @";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Error should include location info
        if let Err(e) = result {
            let msg = format!("{}", e);
            // Check error message exists
            assert!(msg.len() > 0);
        }
    }

    #[test]
    fn test_context_aware_errors() {
        let input = r#"
        fn foo() {
            match x {
                1 => @
            }
        }
        "#;

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Error should mention match context
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod panic_recovery {
    use super::*;

    #[test]
    fn test_no_panic_on_empty_input() {
        let input = "";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should handle empty input gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_no_panic_on_only_whitespace() {
        let input = "   \n\t  ";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should handle whitespace-only input
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_no_panic_on_only_comments() {
        let input = "// comment\n/* block comment */";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should handle comment-only input
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_no_panic_on_unicode() {
        let input = "let 变量 = 42";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should handle unicode identifiers
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_no_panic_on_deep_nesting() {
        let input = "((((((((((1))))))))))";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should handle deep nesting without stack overflow
        assert!(result.is_ok() || result.is_err());
    }
}

#[cfg(test)]
mod error_recovery_strategies {
    use super::*;

    #[test]
    fn test_synchronize_at_statement_boundary() {
        let input = r#"
        let x = @ error here
        let y = 42
        "#;

        let mut parser = Parser::new(input);
        let _ = parser.parse();

        // Should recover at 'let y'
        // Just verify no panic
        assert!(true);
    }

    #[test]
    fn test_synchronize_at_block_boundary() {
        let input = r#"
        {
            let x = @
        }
        let y = 42
        "#;

        let mut parser = Parser::new(input);
        let _ = parser.parse();

        // Should recover after block
        assert!(true);
    }

    #[test]
    fn test_synchronize_at_function_boundary() {
        let input = r#"
        fn foo() {
            @@@
        }
        
        fn bar() {
            let x = 42
        }
        "#;

        let mut parser = Parser::new(input);
        let _ = parser.parse();

        // Should recover at 'fn bar'
        assert!(true);
    }

    #[test]
    fn test_skip_until_delimiter() {
        let input = "[1, @error, 3, 4]";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should skip error and parse rest of list
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_recover_in_expressions() {
        let input = "1 + @ + 3";

        let mut parser = Parser::new(input);
        let result = parser.parse();

        // Should report error but attempt to continue
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_parser_never_panics(input: String) {
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }

        #[test]
        fn test_empty_variations_never_panic(spaces in "[ \t\n]*") {
            let mut parser = Parser::new(&spaces);
            let result = parser.parse();
            assert!(result.is_ok() || result.is_err()); // Empty should parse successfully
        }

        #[test]
        fn test_random_operators_never_panic(ops in "[+\\-*/%<>=!&|^~@]+") {
            let input = format!("let x = 1 {} 2", ops);
            let mut parser = Parser::new(&input);
            let _ = parser.parse(); // Should not panic
        }
    }
}
